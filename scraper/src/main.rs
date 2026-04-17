use filereduce::translations::TranslationConfig;
use filereduce_scraper::EdifactoryScraper;
use serde_json;
use std::env;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

fn load_existing_config(path: &Path) -> Option<TranslationConfig> {
    if path.exists() {
        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => Some(config),
                Err(e) => {
                    eprintln!(
                        "Warning: failed to parse existing config {}: {}",
                        path.display(),
                        e
                    );
                    None
                }
            },
            Err(e) => {
                eprintln!(
                    "Warning: failed to read existing config {}: {}",
                    path.display(),
                    e
                );
                None
            }
        }
    } else {
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: {} <version> [output_dir] [--segment SEGMENT]",
            args[0]
        );
        eprintln!("       {} --all [output_dir] [--force]", args[0]);
        eprintln!("Example: {} D96A standards", args[0]);
        eprintln!("Example: {} D96A standards --segment BGM", args[0]);
        eprintln!("Example: {} --all standards", args[0]);
        std::process::exit(1);
    }

    let mut output_dir = "standards".to_string();
    let mut version = None;
    let mut segment_filter = None;
    let mut scrape_all = false;
    let mut force_overwrite = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--all" => {
                scrape_all = true;
                i += 1;
            }
            "--segment" => {
                if i + 1 < args.len() {
                    segment_filter = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --segment requires a segment code");
                    std::process::exit(1);
                }
            }
            "--force" => {
                force_overwrite = true;
                i += 1;
            }
            arg => {
                // If this is the first non-flag argument and version not set yet
                if version.is_none() && !arg.starts_with("--") && !scrape_all {
                    version = Some(arg.to_string());
                } else if !arg.starts_with("--") {
                    // Assume output directory
                    output_dir = arg.to_string();
                }
                i += 1;
            }
        }
    }

    let scraper = EdifactoryScraper::new();

    if scrape_all {
        println!("Listing all available EDIFACT versions...");
        let versions = scraper.list_versions()?;
        println!("Found {} versions: {:?}", versions.len(), versions);

        let total = versions.len();
        for (idx, version) in versions.iter().enumerate() {
            println!("Processing version {} ({}/{})...", version, idx + 1, total);
            thread::sleep(Duration::from_millis(1000));
            let output_path = Path::new(&output_dir).join(format!("{}.json", version));
            if !force_overwrite && output_path.exists() {
                if let Ok(metadata) = fs::metadata(&output_path) {
                    if metadata.len() > 5000 {
                        println!(
                            "  Skipping version {} (file already exists and appears complete)",
                            version
                        );
                        continue;
                    }
                }
            }
            let existing_config = if force_overwrite {
                None
            } else {
                load_existing_config(&output_path)
            };

            let config = match (existing_config.as_ref(), force_overwrite) {
                (Some(existing), false) => {
                    match scraper.scrape_version_update(&version, Some(existing.clone())) {
                        Ok(config) => config,
                        Err(e) => {
                            eprintln!("  Warning: failed to update version {}: {}", version, e);
                            continue;
                        }
                    }
                }
                _ => match scraper.scrape_version(&version) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("  Warning: failed to scrape version {}: {}", version, e);
                        continue;
                    }
                },
            };

            if let Err(e) = fs::create_dir_all(&output_dir) {
                eprintln!(
                    "  Warning: failed to create directory {}: {}",
                    output_dir, e
                );
                continue;
            }

            let json = match serde_json::to_string_pretty(&config) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!(
                        "  Warning: failed to serialize config for {}: {}",
                        version, e
                    );
                    continue;
                }
            };

            if let Err(e) = fs::write(&output_path, json) {
                eprintln!(
                    "  Warning: failed to write file {}: {}",
                    output_path.display(),
                    e
                );
                continue;
            }
            println!("  Saved translation to {}", output_path.display());
        }
        println!("All versions processed.");
    } else {
        let version = version.ok_or(
            "No version specified. Use --all to scrape all versions or provide a version code.",
        )?;
        println!("Scraping translation for version {}...", version);

        let output_path = Path::new(&output_dir).join(format!("{}.json", version));
        let existing_config = if force_overwrite {
            None
        } else {
            load_existing_config(&output_path)
        };

        let config = if let Some(seg) = segment_filter {
            // Scrape only the specified segment
            let segments = scraper.scrape_segments(&version)?;
            let description = segments
                .iter()
                .find(|(code, _)| *code == seg)
                .map(|(_, desc)| desc.clone())
                .unwrap_or_default();
            let segment_config = scraper.scrape_segment(&version, &seg, &description)?;
            let mut segments_map = std::collections::BTreeMap::new();
            segments_map.insert(seg.to_string(), segment_config);
            filereduce::translations::TranslationConfig {
                version: version.clone(),
                metadata: filereduce::translations::Metadata {
                    last_updated: chrono::Utc::now().to_rfc3339(),
                    description: format!("Single segment translation for {}", seg),
                },
                segments: segments_map,
            }
        } else if let Some(existing) = existing_config {
            scraper.scrape_version_update(&version, Some(existing))?
        } else {
            scraper.scrape_version(&version)?
        };

        fs::create_dir_all(&output_dir)?;
        let json = serde_json::to_string_pretty(&config)?;
        fs::write(&output_path, json)?;
        println!("Saved translation to {}", output_path.display());
    }

    Ok(())
}
