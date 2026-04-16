use filereduce_scraper::EdifactoryScraper;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: {} <version> [output_dir] [--segment SEGMENT]",
            args[0]
        );
        eprintln!("Example: {} D96A standards", args[0]);
        eprintln!("Example: {} D96A standards --segment BGM", args[0]);
        std::process::exit(1);
    }
    let version = args[1].clone();
    let output_dir = args
        .get(2)
        .filter(|&s| !s.starts_with("--"))
        .map(|s| s.as_str())
        .unwrap_or("standards");
    let segment_filter = args
        .iter()
        .position(|a| a == "--segment")
        .and_then(|pos| args.get(pos + 1));

    let scraper = EdifactoryScraper::new();
    println!("Scraping translation for version {}...", version);
    let config = if let Some(seg) = segment_filter {
        // Scrape only the specified segment
        let segments = scraper.scrape_segments(&version)?;
        let description = segments
            .iter()
            .find(|(code, _)| code == seg)
            .map(|(_, desc)| desc.clone())
            .unwrap_or_default();
        let segment_config = scraper.scrape_segment(&version, seg, &description)?;
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
    } else {
        scraper.scrape_version(&version)?
    };

    let output_path = Path::new(output_dir).join(format!("{}.json", version));
    fs::create_dir_all(output_dir)?;
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(&output_path, json)?;
    println!("Saved translation to {}", output_path.display());

    Ok(())
}
