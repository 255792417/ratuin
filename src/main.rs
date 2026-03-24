use anyhow::Result;
use chrono::Utc;
use clap::Parser;

use ratuin::{
    cli::{Cli, Commands},
    db,
    model::HistoryEntry,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let conn = db::open_db()?;

    match cli.command {
        Commands::Record {
            command,
            cwd,
            exit_code,
            duration_ms,
        } => {
            let history_entry = HistoryEntry {
                id: None,
                command,
                cwd,
                exit_code,
                duration_ms,
                timestamp: Utc::now(),
            };

            db::insert_history_entry(&conn, &history_entry)?;
            println!("Recorded command: {}", history_entry);
        }
        Commands::Search { keyword, limit } => {
            let results = db::search_history(&conn, &keyword, limit)?;

            if results.is_empty() {
                println!("No results found for keyword: {}", keyword);
            } else {
                for entry in results {
                    println!("{}", entry);
                }
            }
        }
    }

    Ok(())
}
