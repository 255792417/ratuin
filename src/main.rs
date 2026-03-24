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
            println!("Recorded command: {:#?}", history_entry);
        }
        Commands::Search { keyword } => {
            let results = db::search_history(&conn, &keyword)?;

            for entry in results {
                let time_passed_secs = Utc::now()
                    .signed_duration_since(entry.timestamp)
                    .num_seconds();
                let time_passed = if time_passed_secs < 60 {
                    format!("{}s ago", time_passed_secs)
                } else if time_passed_secs < 3600 {
                    format!("{}m ago", time_passed_secs / 60)
                } else if time_passed_secs < 86400 {
                    format!("{}h ago", time_passed_secs / 3600)
                } else {
                    format!("{}d ago", time_passed_secs / 86400)
                };

                let id = entry.id.unwrap_or(-1);
                println!("{} [{}] {}", time_passed, id, entry.command);
            }
        }
    }

    Ok(())
}
