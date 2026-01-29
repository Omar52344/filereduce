use clap::Parser;
use engine_filereduce::query::parser::Parser as QueryParser;
use filereduce::cli::{Cli, Commands};
use filereduce::error::Result;
use filereduce::processor::{process, FileFormat};
use std::fs::File;
use std::io::{BufReader, BufWriter};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt().init();
    }

    match cli.command {
        Commands::Process {
            input,
            output,
            format,
            query,
            limit: _,
            fra,
        } => {
            let input_file = File::open(&input)?;
            let output_file = File::create(&output)?;

            let file_format = determine_format(&input, format.as_deref());

            let expr = if let Some(q) = query {
                let mut parser = QueryParser::new(&q);
                Some(parser.parse())
            } else {
                None
            };

            let mut sink = filereduce::sink::file::FileDataSink::new(BufWriter::new(output_file));

            process(
                BufReader::new(input_file),
                &mut sink,
                file_format,
                expr.as_ref(),
            )
            .await?;

            use filereduce::sink::DataSink; // Import trait
            sink.flush().await?;

            println!("Processed {} to {}", input.display(), output.display());

            if fra {
                println!("Compressing to .fra...");
                let input_jsonl = File::open(&output)?;
                let fra_path = output.with_extension("fra");
                let output_fra = File::create(&fra_path)?;

                let mut compressor = filereducelib::FileReduceCompressor::new();
                compressor
                    .compress(input_jsonl, output_fra)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                println!("Compressed to {}", fra_path.display());
            }
        }

        Commands::Insert { input, config, fra } => {
            let config_content = std::fs::read_to_string(&config)?;
            let ingest_config: filereduce::config::IngestConfig =
                serde_yaml::from_str(&config_content)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            let mut sink = filereduce::sink::db::DbDataSink::new(ingest_config.ingest).await?;
            let input_file = File::open(&input)?;

            use filereduce::sink::DataSink;
            process(
                BufReader::new(input_file),
                &mut sink,
                FileFormat::Edifact,
                None,
            )
            .await?;
            sink.flush().await?;

            if fra {
                println!("Generating optional .fra output...");
                let temp_path = input.with_extension("tmp.jsonl");

                {
                    let temp_file = File::create(&temp_path)?;
                    let mut temp_sink =
                        filereduce::sink::file::FileDataSink::new(BufWriter::new(temp_file));
                    let input_file_2 = File::open(&input)?;

                    process(
                        BufReader::new(input_file_2),
                        &mut temp_sink,
                        FileFormat::Edifact,
                        None,
                    )
                    .await?;
                    temp_sink.flush().await?;
                }

                let fra_path = input.with_extension("fra");
                let input_jsonl = File::open(&temp_path)?;
                let output_fra = File::create(&fra_path)?;

                let mut compressor = filereducelib::FileReduceCompressor::new();
                compressor
                    .compress(input_jsonl, output_fra)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

                std::fs::remove_file(temp_path)?;
                println!("Compressed to {}", fra_path.display());
            }
        }

        Commands::Query {
            input,
            query: _,
            output: _,
        } => {
            println!(
                "Query command not fully implemented in async refactor yet for {}",
                input.display()
            );
        }

        Commands::Convert {
            input,
            output: _,
            from,
            to,
        } => {
            println!("Convert {} from {} to {}", input.display(), from, to);
        }
    }

    Ok(())
}

fn determine_format(path: &std::path::Path, format: Option<&str>) -> FileFormat {
    if let Some(fmt) = format {
        match fmt.to_lowercase().as_str() {
            "xml" => FileFormat::Xml,
            "json" => FileFormat::Json,
            _ => FileFormat::Edifact,
        }
    } else {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("xml") => FileFormat::Xml,
            Some("json") | Some("jsonl") => FileFormat::Json,
            _ => FileFormat::Edifact,
        }
    }
}

#[test]
fn parse_simple_where() {
    use engine_filereduce::query::ast::Expr;
    use engine_filereduce::query::parser::Parser;

    let mut parser = Parser::new("kind = 'LIN' AND qty > 5");
    let expr = parser.parse();

    match expr {
        Expr::And(_, _) => {}
        _ => panic!("No se construyó un AND"),
    }
}

#[test]
fn parse_like_operator() {
    use engine_filereduce::query::ast::Expr;
    use engine_filereduce::query::parser::Parser;

    let mut parser = Parser::new("sku LIKE 'ABC%'");
    let expr = parser.parse();

    match expr {
        Expr::Like(_, _) => {}
        _ => panic!("No se construyó un LIKE"),
    }
}

#[test]
fn parse_in_operator() {
    use engine_filereduce::query::ast::Expr;
    use engine_filereduce::query::parser::Parser;

    let mut parser = Parser::new("qty IN (1, 2, 3)");
    let expr = parser.parse();

    match expr {
        Expr::In(_, _) => {}
        _ => panic!("No se construyó un IN"),
    }
}

#[test]
fn parse_between_operator() {
    use engine_filereduce::query::ast::Expr;
    use engine_filereduce::query::parser::Parser;

    let mut parser = Parser::new("qty BETWEEN 1 AND 10");
    let expr = parser.parse();

    match expr {
        Expr::Between(_, _, _) => {}
        _ => panic!("No se construyó un BETWEEN"),
    }
}

#[test]
fn parse_or_operator() {
    use engine_filereduce::query::ast::Expr;
    use engine_filereduce::query::parser::Parser;

    let mut parser = Parser::new("kind = 'LIN' OR kind = 'BGM'");
    let expr = parser.parse();

    match expr {
        Expr::Or(_, _) => {}
        _ => panic!("No se construyó un OR"),
    }
}
