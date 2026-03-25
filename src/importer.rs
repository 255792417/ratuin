use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;
use std::{fs, path::PathBuf};

use crate::{db, importer_parsers, model::HistoryEntry, privacy};

#[derive(Debug)]
pub struct ImportStats {
    pub source: PathBuf,
    pub imported: usize,
    pub skipped_empty: usize,
    pub skipped_sensitive: usize,
    pub skipped_duplicate: usize,
}

fn resolve_history_source(shell: &str, file: Option<PathBuf>) -> Result<PathBuf> {
    match file {
        Some(path) => Ok(path),
        None => importer_parsers::default_history_path(shell),
    }
}

fn import_record(
    conn: &Connection,
    command: String,
    timestamp: chrono::DateTime<Utc>,
    duration_ms: u64,
) -> Result<bool> {
    let entry = HistoryEntry {
        id: None,
        command,
        cwd: "imported".to_string(),
        exit_code: 0,
        duration_ms,
        timestamp,
    };

    if db::history_entry_exists(conn, &entry)? {
        return Ok(false);
    }

    db::insert_history_entry(conn, &entry)?;
    Ok(true)
}

pub fn import_history_file(
    conn: &Connection,
    shell: &str,
    file: Option<PathBuf>,
    allow_sensitive: bool,
) -> Result<ImportStats> {
    let kind = importer_parsers::ShellKind::from_str(shell)?;
    let source = resolve_history_source(shell, file)?;

    let content = fs::read_to_string(&source)?;
    let parsed = importer_parsers::parse_history_by_shell(kind, &content);

    conn.execute_batch("BEGIN TRANSACTION;")?;

    let import_result: Result<ImportStats> = (|| {
        let mut stats = ImportStats {
            source,
            imported: 0,
            skipped_empty: 0,
            skipped_sensitive: 0,
            skipped_duplicate: 0,
        };

        for record in parsed {
            let command = record.command.trim().to_string();
            if command.is_empty() {
                stats.skipped_empty += 1;
                continue;
            }

            if !allow_sensitive && privacy::is_sensitive_command(&command) {
                stats.skipped_sensitive += 1;
                continue;
            }

            let imported = import_record(
                conn,
                command,
                record.timestamp.unwrap_or_else(Utc::now),
                record.duration_ms,
            )?;

            if !imported {
                stats.skipped_duplicate += 1;
                continue;
            }

            stats.imported += 1;
        }

        Ok(stats)
    })();

    match import_result {
        Ok(stats) => {
            conn.execute_batch("COMMIT;")?;
            Ok(stats)
        }
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK;");
            Err(error)
        }
    }
}
