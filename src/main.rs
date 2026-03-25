use anyhow::Result;
use chrono::Utc;
use clap::Parser;

use ratuin::{
    cli::{Cli, Commands},
    db, importer,
    model::HistoryEntry,
    privacy,
};

const SUPPORTED_SHELLS: [&str; 3] = ["bash", "zsh", "fish"];

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
        Commands::Search { .. } => {}
        Commands::Import { shell, file, .. } => {
            if !SUPPORTED_SHELLS.contains(&shell.as_str()) {
                anyhow::bail!(
                    "Unsupported shell: {}. Supported shells are: {}",
                    shell,
                    SUPPORTED_SHELLS.join(", ")
                );
            }

            if let Some(path) = file
                && !path.exists()
            {
                anyhow::bail!("History file does not exist: {}", path.display());
            }
        }
        Commands::Init { shell } => {
            if !SUPPORTED_SHELLS.contains(&shell.as_str()) {
                anyhow::bail!(
                    "Unsupported shell: {}. Supported shells are: {}",
                    shell,
                    SUPPORTED_SHELLS.join(", ")
                );
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
        Commands::Init { shell } => {
            let script = match shell.as_str() {
                "bash" => std::include_str!("scripts/init.bash"),
                "zsh" => std::include_str!("scripts/init.zsh"),
                "fish" => std::include_str!("scripts/init.fish"),
                _ => anyhow::bail!("Unsupported shell: {}", shell),
            };

            println!("{}", script);
        }
        Commands::Import {
            shell,
            file,
            allow_sensitive,
        } => {
            let stats = importer::import_history_file(&conn, &shell, file, allow_sensitive)?;

            println!(
                "Imported {} entries from {} (skipped empty: {}, skipped sensitive: {})",
                stats.imported,
                stats.source.display(),
                stats.skipped_empty,
                stats.skipped_sensitive
            );
        }
        Commands::Record {
            command,
            cwd,
            exit_code,
            duration_ms,
            allow_sensitive,
        } => {
            if !allow_sensitive && privacy::is_sensitive_command(&command) {
                println!("Skipped sensitive command");
                return Ok(());
            }

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
