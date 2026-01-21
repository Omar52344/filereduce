use clap::Parser;
use filereduce::cli::{Cli, Commands};
use filereduce::error::Result;
use filereduce::processor::process;
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        tracing_subscriber::fmt().init();
    }

    match cli.command {
        Commands::Process {
            input,
            output,
            format: _,
            query: _,
            limit: _,
        } => {
            let input_file = File::open(&input)?;
            let output_file = File::create(&output)?;

            process(BufReader::new(input_file), &mut BufWriter::new(output_file))?;
            println!("Processed {} to {}", input.display(), output.display());
        }

        Commands::Query {
            input,
            query,
            output,
        } => {
            println!("Query: {} on {}", query, input.display());
            if let Some(out) = output {
                println!("Output: {}", out.display());
            }
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
