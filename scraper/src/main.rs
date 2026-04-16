use filereduce_scraper::EdifactoryScraper;
use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let version = if args.len() > 1 {
        args[1].clone()
    } else {
        eprintln!("Usage: {} <version> [output_dir]", args[0]);
        eprintln!("Example: {} D96A standards", args[0]);
        std::process::exit(1);
    };
    let output_dir = args.get(2).map(|s| s.as_str()).unwrap_or("standards");

    let scraper = EdifactoryScraper::new();
    println!("Scraping translation for version {}...", version);
    let config = scraper.scrape_version(&version)?;

    let output_path = Path::new(output_dir).join(format!("{}.json", version));
    fs::create_dir_all(output_dir)?;
    let json = serde_json::to_string_pretty(&config)?;
    fs::write(&output_path, json)?;
    println!("Saved translation to {}", output_path.display());

    Ok(())
}
