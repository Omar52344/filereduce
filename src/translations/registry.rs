use crate::error::{FileReduceError, Result};
use crate::translations::config::*;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::RwLock;

#[derive(Debug)]
pub struct TranslationRegistry {
    config: RwLock<TranslationConfig>,
}

impl TranslationRegistry {
    pub fn new() -> Result<Self> {
        let config = Self::load_default()?;
        Ok(Self {
            config: RwLock::new(config),
        })
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: TranslationConfig =
            serde_json::from_str(&content).map_err(|e| FileReduceError::Serialization(e))?;
        Ok(Self {
            config: RwLock::new(config),
        })
    }

    /// Load translation configuration for a specific EDIFACT version
    /// Looks for file at `standards/{version}.json` relative to current directory
    pub fn from_version(version: &str) -> Result<Self> {
        let path = format!("standards/{}.json", version);
        Self::from_file(path)
    }

    /// Load translation configuration for a specific EDIFACT version, scraping if missing.
    /// Looks for file at `standards/{version}.json` relative to current directory.
    /// If file does not exist, attempts to run the scraper binary to generate it.
    pub fn from_version_or_scrape(version: &str) -> Result<Self> {
        let path = format!("standards/{}.json", version);
        if Path::new(&path).exists() {
            return Self::from_file(path);
        }
        // Attempt to scrape
        eprintln!(
            "Translation file for version {} not found, attempting to scrape...",
            version
        );
        let scraper_bin_candidates = vec![
            "filereduce-scraper".to_string(),
            "./scraper/target/release/filereduce-scraper".to_string(),
            "./target/release/filereduce-scraper".to_string(),
            "../target/release/filereduce-scraper".to_string(),
        ];
        let mut last_error = None;
        for bin in scraper_bin_candidates {
            match Command::new(&bin).arg(version).arg("standards").output() {
                Ok(output) if output.status.success() => {
                    // success, break and load file
                    return Self::from_file(path);
                }
                Ok(output) => {
                    // scraper ran but failed
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    last_error = Some(format!("Scraper {} failed: {}", bin, stderr));
                    // continue to next candidate? Probably same error, break.
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // try next candidate
                    continue;
                }
                Err(e) => {
                    return Err(FileReduceError::Io(e));
                }
            }
        }
        return Err(FileReduceError::Parse(last_error.unwrap_or_else(|| {
            "Scraper not found and no candidate succeeded".to_string()
        })));
    }

    fn load_default() -> Result<TranslationConfig> {
        let content = include_str!("../../translations.json");
        serde_json::from_str(content).map_err(|e| FileReduceError::Serialization(e))
    }

    pub fn get_segment(&self, segment_code: &str) -> Option<SegmentConfig> {
        let config = self.config.read().unwrap();
        config.segments.get(segment_code).cloned()
    }

    pub fn get_qualifier(&self, segment_code: &str, qualifier: &str) -> Option<SubSegmentConfig> {
        let config = self.config.read().unwrap();
        config.segments.get(segment_code).and_then(|seg| {
            if seg.use_qualifier {
                seg.qualifiers.get(qualifier).cloned()
            } else {
                None
            }
        })
    }

    pub fn get_element(&self, segment_code: &str, element_pos: &str) -> Option<ElementConfig> {
        let config = self.config.read().unwrap();
        config
            .segments
            .get(segment_code)
            .and_then(|seg| seg.elements.get(element_pos).cloned())
    }

    pub fn reload_from_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let new_config: TranslationConfig =
            serde_json::from_str(&content).map_err(|e| FileReduceError::Serialization(e))?;
        let mut config = self.config.write().unwrap();
        *config = new_config;
        Ok(())
    }
}

impl Clone for TranslationRegistry {
    fn clone(&self) -> Self {
        let config = self.config.read().unwrap().clone();
        Self {
            config: RwLock::new(config),
        }
    }
}

impl Default for TranslationRegistry {
    fn default() -> Self {
        Self::new().expect("Failed to load default translation config")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_default() {
        let registry = TranslationRegistry::new();
        assert!(registry.is_ok());
        let registry = registry.unwrap();

        // Check that some expected segments are loaded
        let bgm = registry.get_segment("BGM");
        assert!(bgm.is_some());
        let bgm_config = bgm.unwrap();
        assert_eq!(bgm_config.label, "DocumentHeader");
        assert!(bgm_config.elements.contains_key("1"));

        // Check qualifier segments
        let dtm = registry.get_segment("DTM");
        assert!(dtm.is_some());
        let dtm_config = dtm.unwrap();
        assert!(dtm_config.use_qualifier);
        assert!(dtm_config.qualifiers.contains_key("137"));

        // Check element mapping
        let element = registry.get_element("BGM", "1");
        assert!(matches!(element, Some(ElementConfig::Simple(_))));
    }
}
