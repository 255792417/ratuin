use anyhow::Result;
use chrono::Utc;
use clap::Parser;

use ratuin::{
    cli::{Cli, Commands},
    db,
    model::HistoryEntry,
};

fn get_cwd(cwd: Option<String>) -> String {
    if let Some(cwd) = cwd
        && !cwd.is_empty()
    {
        return cwd;
    } else {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

fn validate_command(command: &Commands) -> Result<()> {
    match command {
        Commands::Record { command, .. } => {
            if command.trim().is_empty() {
                anyhow::bail!("Command cannot be empty");
            }
        }
        Commands::Search { keyword, .. } => {
            if keyword.trim().is_empty() {
                anyhow::bail!("Search keyword cannot be empty");
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    validate_command(&cli.command)?;

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
                cwd: get_cwd(cwd),
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
