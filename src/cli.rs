use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "ratuin")]
#[command(version)]
#[command(about = "A tiny atuin-like shell history manager", long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init {
        shell: String,
    },
    Import {
        #[arg(long)]
        shell: String,

        #[arg(long)]
        file: Option<PathBuf>,

        #[arg(long, default_value_t = false)]
        allow_sensitive: bool,
    },
    Record {
        #[arg(long)]
        command: String,

        #[arg(long)]
        cwd: Option<String>,

        #[arg(long, default_value_t = 0)]
        exit_code: u32,

        #[arg(long, default_value_t = 0)]
        duration_ms: u64,

        #[arg(long, default_value_t = false)]
        allow_sensitive: bool,
    },
    Search {
        keyword: String,

        #[arg(long)]
        limit: Option<usize>,
    },
}
