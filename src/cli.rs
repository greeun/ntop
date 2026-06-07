// CLI argument parsing

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "ntop", version, about = "Node Top - Monitor and manage server processes (Node, Python, Java, and more)")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    List {
        #[arg(long)]
        json: bool,
        #[arg(long)]
        format: Option<ListFormat>,
    },
    Kill {
        #[arg(required_unless_present = "all")]
        pid: Option<u32>,
        #[arg(long)]
        tree: bool,
        #[arg(long)]
        signal: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        no_confirm: bool,
    },
    Info { pid: u32 },
    Log { pid: u32 },
    Config,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum ListFormat {
    Table,
    Csv,
    Json,
}
