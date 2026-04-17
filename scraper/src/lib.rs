use filereduce::translations::{ElementConfig, Metadata, SegmentConfig, TranslationConfig};
use regex::Regex;
use scraper::{Html, Selector};
use std::collections::{BTreeMap, HashMap};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Parsing error: {0}")]
    Parse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Selector error: {0}")]
    Selector(String),
}

pub struct EdifactoryScraper {
    client: reqwest::blocking::Client,
    base_url: String,
}

impl EdifactoryScraper {
    pub fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
            base_url: "https://www.edifactory.de/edifact".to_string(),
        }
    }

    pub fn list_versions(&self) -> Result<Vec<String>, ScraperError> {
        let url = format!("{}/directories", self.base_url);
        let resp = self.client.get(&url).send()?;
        let body = resp.text()?;

        // Extract version patterns like directory/D96A or directory/D01B
        let re = Regex::new(r#"directory/(D\d{2,3}[A-Z]?)"#).unwrap();
        let mut versions: Vec<String> = re
            .captures_iter(&body)
            .map(|cap| cap[1].to_string())
            .collect();

        // Also look for links that are just version directories (D96A/)
        let re2 = Regex::new(r#"href="([^"]*?)(D\d{2,3}[A-Z]?)/"#).unwrap();
        for cap in re2.captures_iter(&body) {
            versions.push(cap[2].to_string());
        }

        versions.sort();
        versions.dedup();
        Ok(versions)
    }

    pub fn scrape_version(&self, version: &str) -> Result<TranslationConfig, ScraperError> {
        let segments = self.scrape_segments(version)?;

        // Common segments used in typical EDIFACT messages
        let whitelist: std::collections::HashSet<&'static str> = [
            "BGM", "DTM", "NAD", "LIN", "QTY", "PRI", "RFF", "CTA", "LOC", "TDT", "PAC", "MEA",
            "ALC", "TAX", "MOA", "CUX", "FTX", "DOC", "UNH", "UNT", "UNB",
            "UNZ", // service segments
        ]
        .into_iter()
        .collect();
        let mut segment_configs = BTreeMap::new();
        for (code, description) in segments {
            if whitelist.contains(&code.as_str()) {
                if let Ok(segment_config) = self.scrape_segment(version, &code, &description) {
                    segment_configs.insert(code, segment_config);
                } else {
                    eprintln!("Failed to scrape segment {}", code);
                }
            }
        }

        Ok(TranslationConfig {
            version: version.to_string(),
            metadata: Metadata {
                last_updated: chrono::Utc::now().to_rfc3339(),
                description: format!("Auto‑scraped translation for EDIFACT version {}", version),
            },
            segments: segment_configs,
        })
    }

    pub fn scrape_version_update(
        &self,
        version: &str,
        existing: Option<TranslationConfig>,
    ) -> Result<TranslationConfig, ScraperError> {
        let segments = self.scrape_segments(version)?;

        // Common segments used in typical EDIFACT messages
        let whitelist: std::collections::HashSet<&'static str> = [
            "BGM", "DTM", "NAD", "LIN", "QTY", "PRI", "RFF", "CTA", "LOC", "TDT", "PAC", "MEA",
            "ALC", "TAX", "MOA", "CUX", "FTX", "DOC", "UNH", "UNT", "UNB",
            "UNZ", // service segments
        ]
        .into_iter()
        .collect();

        // Start with existing segments if provided
        let mut segment_configs = existing.map(|c| c.segments).unwrap_or_else(BTreeMap::new);

        for (code, description) in segments {
            if whitelist.contains(&code.as_str()) {
                // Only scrape if not already present (avoid duplicates)
                if !segment_configs.contains_key(&code) {
                    if let Ok(segment_config) = self.scrape_segment(version, &code, &description) {
                        segment_configs.insert(code, segment_config);
                    } else {
                        eprintln!("Failed to scrape segment {}", code);
                    }
                }
            }
        }

        Ok(TranslationConfig {
            version: version.to_string(),
            metadata: Metadata {
                last_updated: chrono::Utc::now().to_rfc3339(),
                description: format!(
                    "Auto‑scraped translation for EDIFACT version {} (updated)",
                    version
                ),
            },
            segments: segment_configs,
        })
    }

    pub fn scrape_segments(&self, version: &str) -> Result<Vec<(String, String)>, ScraperError> {
        let url = format!("{}/directory/{}/segments", self.base_url, version);
        let resp = self.client.get(&url).send()?;
        let body = resp.text()?;
        let doc = Html::parse_document(&body);

        let row_selector =
            Selector::parse("table tbody tr").map_err(|e| ScraperError::Selector(e.to_string()))?;
        let code_selector = Selector::parse("td:first-child a")
            .map_err(|e| ScraperError::Selector(e.to_string()))?;
        let desc_selector =
            Selector::parse("td.unbold").map_err(|e| ScraperError::Selector(e.to_string()))?;

        let mut segments = Vec::new();
        for row in doc.select(&row_selector) {
            let code_elem = row.select(&code_selector).next();
            let desc_elem = row.select(&desc_selector).next();
            if let (Some(code), Some(desc)) = (code_elem, desc_elem) {
                let code = code.text().collect::<String>().trim().to_string();
                let desc = desc
                    .text()
                    .collect::<String>()
                    .replace('\n', " ")
                    .trim()
                    .to_string();
                if !code.is_empty() {
                    segments.push((code, desc));
                }
            }
        }
        Ok(segments)
    }

    pub fn scrape_segment(
        &self,
        version: &str,
        code: &str,
        description: &str,
    ) -> Result<SegmentConfig, ScraperError> {
        let url = format!("{}/directory/{}/segment/{}", self.base_url, version, code);
        let resp = self.client.get(&url).send()?;
        let body = resp.text()?;
        let doc = Html::parse_document(&body);

        // Extract the pre block containing the segment structure
        let pre_selector = Selector::parse("div.segment-content pre")
            .map_err(|e| ScraperError::Selector(e.to_string()))?;
        let pre = doc
            .select(&pre_selector)
            .next()
            .ok_or_else(|| ScraperError::Parse("Segment structure not found".to_string()))?;
        let structure_text = pre.text().collect::<String>();

        // Parse the structure into elements
        let (use_qualifier, elements) = self.parse_segment_structure(&structure_text, code)?;

        // Determine label: use the first line of description (before first period or "To specify")
        let label = description
            .split('.')
            .next()
            .unwrap_or(description)
            .split(" To ")
            .next()
            .unwrap_or(description)
            .trim()
            .to_string();
        // Capitalize first letter
        let label = if let Some(first) = label.get(0..1) {
            first.to_uppercase() + &label[1..]
        } else {
            label
        };

        Ok(SegmentConfig {
            label,
            use_qualifier,
            qualifiers: BTreeMap::new(), // TODO: scrape qualifier codes
            elements,
        })
    }

    fn parse_segment_structure(
        &self,
        text: &str,
        segment_code: &str,
    ) -> Result<(bool, BTreeMap<String, ElementConfig>), ScraperError> {
        // Remove HTML tags
        let re = Regex::new(r"<[^>]*>").unwrap();
        let clean_text = re.replace_all(text, "");
        let lines: Vec<&str> = clean_text.lines().collect();
        eprintln!("=== Parsing segment structure ===");
        for (idx, line) in lines.iter().enumerate() {
            eprintln!("{}: '{}'", idx, line);
        }

        let mut elements = BTreeMap::new();
        let mut use_qualifier = false;
        let mut position = 1;
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.is_empty() {
                i += 1;
                continue;
            }
            // Determine indentation: count leading spaces (original line before trim)
            let original = lines[i];
            let indent = original.len() - original.trim_start().len();
            eprintln!("Processing line {} indent {}: '{}'", i, indent, line);
            if indent == 0 {
                // Top-level element (segment position)
                // Check if line starts with a composite tag (like C002) or a simple data element
                if line.starts_with(|c: char| c.is_ascii_uppercase())
                    && line.chars().nth(1).map(|c| c.is_digit(10)).unwrap_or(false)
                {
                    eprintln!("  Composite detected at position {}", position);
                    // Composite element line, e.g., "C002       DOCUMENT/MESSAGE NAME                              C"
                    // The following indented lines (indent 3) are its components
                    let mut component_lines = Vec::new();
                    let mut j = i + 1;
                    while j < lines.len() {
                        let next_original = lines[j];
                        let next_indent = next_original.len() - next_original.trim_start().len();
                        if next_indent >= 3 {
                            component_lines.push(next_original.trim());
                            j += 1;
                        } else {
                            break;
                        }
                    }
                    eprintln!("  Components: {:?}", component_lines);
                    // Parse components
                    let mut components = BTreeMap::new();
                    let mut comp_pos = 1;
                    for comp_line in component_lines {
                        eprintln!("    Component line: '{}'", comp_line);
                        if let Some(desc) = self.extract_description_from_component(comp_line) {
                            eprintln!("      -> desc '{}' at pos {}", desc, comp_pos);
                            components.insert(comp_pos.to_string(), desc);
                            comp_pos += 1;
                        } else {
                            eprintln!("      -> no description extracted");
                        }
                    }
                    let raw_label = self.extract_label_from_line(line);
                    let label = self.normalize_element_label(segment_code, position, &raw_label);
                    elements.insert(
                        position.to_string(),
                        ElementConfig::Composite { label, components },
                    );
                    i = j;
                } else {
                    eprintln!("  Simple element at position {}: {}", position, line);
                    // Simple data element line, e.g., "1004       DOCUMENT/MESSAGE NUMBER                            C   an1..35"
                    let raw_label = self.extract_label_from_line(line);
                    let label = self.normalize_element_label(segment_code, position, &raw_label);
                    // Check if label contains "qualifier" and it's the first element
                    if raw_label.to_lowercase().contains("qualifier") && position == 1 {
                        use_qualifier = true;
                    }
                    elements.insert(position.to_string(), ElementConfig::Simple(label));
                    i += 1;
                }
                position += 1;
            } else {
                // Indented line that belongs to previous composite but we already collected them above
                // Skip (should not happen because we advanced i already)
                eprintln!("  Skipping indented line at i={}", i);
                i += 1;
            }
        }
        eprintln!("Result elements: {:?}", elements.keys().collect::<Vec<_>>());
        Ok((use_qualifier, elements))
    }

    fn extract_description_from_component(&self, line: &str) -> Option<String> {
        // Line format: "1001    Document/message name, coded                       C   an1..3"
        // Extract the descriptive part before "C" or "M"
        self.extract_description(line)
    }

    fn extract_label_from_line(&self, line: &str) -> String {
        // Extract the descriptive part, e.g., "DOCUMENT/MESSAGE NAME"
        self.extract_description(line)
            .unwrap_or_else(|| "Unknown".to_string())
    }

    fn is_format_specifier(&self, part: &str) -> bool {
        part == "C"
            || part == "M"
            || part == "an1..3"
            || (part.starts_with("an")
                && part[2..]
                    .chars()
                    .next()
                    .map(|c| c.is_digit(10))
                    .unwrap_or(false))
            || (part.starts_with("a")
                && part[1..]
                    .chars()
                    .next()
                    .map(|c| c.is_digit(10))
                    .unwrap_or(false))
    }

    fn extract_description(&self, line: &str) -> Option<String> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        // Find the first part that is not a number (or contains letters)
        let mut start = 0;
        for (idx, &part) in parts.iter().enumerate() {
            // Skip if part is all digits (maybe with punctuation like "1001")
            if part.chars().all(|c| c.is_digit(10)) {
                continue;
            }
            // Also skip if part is like "C", "M", "an1..3", etc.
            if self.is_format_specifier(part) {
                continue;
            }
            start = idx;
            break;
        }
        // Collect parts until we hit "C", "M", or an "an..." pattern
        let mut desc_parts = Vec::new();
        for &part in parts.iter().skip(start) {
            if part == "C"
                || part == "M"
                || part == "an1..3"
                || part.starts_with("an")
                || part.starts_with("a")
            {
                break;
            }
            desc_parts.push(part);
        }
        if desc_parts.is_empty() {
            None
        } else {
            Some(desc_parts.join(" ").trim().to_string())
        }
    }

    fn normalize_element_label(
        &self,
        segment_code: &str,
        position: usize,
        raw_label: &str,
    ) -> String {
        // Apply special mappings for known segments
        match (segment_code, position) {
            ("BGM", 2) => "DocumentNumber".to_string(),
            ("BGM", 1) => "MessageName".to_string(),
            ("BGM", 3) => "MessageFunction".to_string(),
            ("DTM", 1) if raw_label.to_lowercase().contains("qualifier") => "Value".to_string(),
            _ => {
                // Generic normalization: convert to title case, replace slashes, remove punctuation
                let mut label = raw_label.to_lowercase();
                // Replace "/" with space
                label = label.replace("/", " ");
                // Remove commas, periods, hyphens
                label = label.replace(",", "").replace(".", "").replace("-", " ");
                // Split into words, capitalize each word, join without spaces
                let words: Vec<String> = label
                    .split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().chain(chars).collect(),
                        }
                    })
                    .collect();
                words.join("")
            }
        }
    }

    /// Merge two translation configs, preferring existing segments (to avoid overwriting)
    pub fn merge_configs(existing: TranslationConfig, new: TranslationConfig) -> TranslationConfig {
        // Ensure versions match
        assert_eq!(existing.version, new.version);

        let mut merged_segments = existing.segments;
        for (code, new_segment) in new.segments {
            // Only insert if not already present (avoid duplicates)
            if !merged_segments.contains_key(&code) {
                merged_segments.insert(code, new_segment);
            }
        }

        TranslationConfig {
            version: existing.version,
            metadata: new.metadata, // Use newer metadata
            segments: merged_segments,
        }
    }

    fn mock_translation_config(&self, version: &str) -> TranslationConfig {
        TranslationConfig {
            version: version.to_string(),
            metadata: Metadata {
                last_updated: chrono::Utc::now().to_rfc3339(),
                description: format!("Mock translation for EDIFACT version {}", version),
            },
            segments: BTreeMap::from_iter([
                (
                    "BGM".to_string(),
                    SegmentConfig {
                        label: "DocumentHeader".to_string(),
                        use_qualifier: false,
                        qualifiers: BTreeMap::new(),
                        elements: BTreeMap::from_iter([
                            (
                                "1".to_string(),
                                ElementConfig::Simple("MessageName".to_string()),
                            ),
                            (
                                "2".to_string(),
                                ElementConfig::Simple("DocumentNumber".to_string()),
                            ),
                        ]),
                    },
                ),
                (
                    "DTM".to_string(),
                    SegmentConfig {
                        label: "DateTime".to_string(),
                        use_qualifier: true,
                        qualifiers: BTreeMap::from_iter([(
                            "137".to_string(),
                            crate::SubSegmentConfig {
                                label: "DocumentDate".to_string(),
                                elements: BTreeMap::from_iter([(
                                    "1".to_string(),
                                    ElementConfig::Simple("Value".to_string()),
                                )]),
                            },
                        )]),
                        elements: BTreeMap::new(),
                    },
                ),
            ]),
        }
    }
}

// Re-export SubSegmentConfig for convenience
pub use filereduce::translations::SubSegmentConfig;
