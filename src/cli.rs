use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "filereduce")]
#[command(about = "EDIFACT/XML/JSON file processor with SQL-like queries", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Process {
        input: PathBuf,
        output: PathBuf,

        #[arg(short = 'f', long)]
        format: Option<String>,

        #[arg(short = 'q', long)]
        query: Option<String>,

        #[arg(short, long, default_value_t = 10000)]
        limit: usize,

        #[arg(long)]
        fra: bool,
    },

    Query {
        input: PathBuf,
        #[arg(short = 'q', long)]
        query: String,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    Convert {
        input: PathBuf,
        output: PathBuf,

        #[arg(short, long)]
        from: String,

        #[arg(short, long)]
        to: String,
    },

    Insert {
        input: PathBuf,
        #[arg(short, long)]
        config: PathBuf,

        #[arg(long)]
        fra: bool,
    },
}
