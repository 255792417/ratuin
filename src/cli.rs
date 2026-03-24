use clap::{Parser, Subcommand};

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
    Record {
        #[arg(long)]
        command: String,

        #[arg(long)]
        cwd: Option<String>,

        #[arg(long, default_value_t = 0)]
        exit_code: i32,

        #[arg(long, default_value_t = 0)]
        duration_ms: i64,
    },
    Search {
        keyword: String,

        #[arg(long)]
        limit: Option<usize>,
    },
}
