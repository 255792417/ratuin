use anyhow::{Result, anyhow};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::Connection;
use std::{fs, path::PathBuf};

use crate::{db, model::HistoryEntry, privacy};

#[derive(Debug)]
struct ImportRecord {
    command: String,
    timestamp: Option<DateTime<Utc>>,
    duration_ms: u64,
}

#[derive(Debug)]
pub struct ImportStats {
    pub source: PathBuf,
    pub imported: usize,
    pub skipped_empty: usize,
    pub skipped_sensitive: usize,
    pub skipped_duplicate: usize,
}

enum ShellKind {
    Bash,
    Zsh,
    Fish,
}

impl ShellKind {
    fn from_str(shell: &str) -> Result<Self> {
        match shell {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "fish" => Ok(Self::Fish),
            _ => Err(anyhow!("Unsupported shell: {shell}")),
        }
    }
}

fn parse_unix_timestamp(timestamp: i64) -> Option<DateTime<Utc>> {
    Utc.timestamp_opt(timestamp, 0).single()
}

fn parse_bash_history(content: &str) -> Vec<ImportRecord> {
    let mut records = Vec::new();
    let mut pending_timestamp: Option<DateTime<Utc>> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(ts_line) = trimmed.strip_prefix('#') {
            if !ts_line.is_empty() && ts_line.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(unix_ts) = ts_line.parse::<i64>() {
                    pending_timestamp = parse_unix_timestamp(unix_ts);
                    continue;
                }
            }
        }

        records.push(ImportRecord {
            command: line.to_string(),
            timestamp: pending_timestamp.take(),
            duration_ms: 0,
        });
    }

    records
}

fn parse_zsh_history(content: &str) -> Vec<ImportRecord> {
    let mut records = Vec::new();

    for line in content.lines() {
        if let Some(rest) = line.strip_prefix(": ")
            && let Some((meta, cmd)) = rest.split_once(';')
        {
            let mut parts = meta.split(':');
            let timestamp = parts
                .next()
                .and_then(|value| value.parse::<i64>().ok())
                .and_then(parse_unix_timestamp);
            let duration_ms = parts
                .next()
                .and_then(|value| value.parse::<u64>().ok())
                .map(|seconds| seconds.saturating_mul(1000))
                .unwrap_or(0);

            records.push(ImportRecord {
                command: cmd.to_string(),
                timestamp,
                duration_ms,
            });
            continue;
        }

        records.push(ImportRecord {
            command: line.to_string(),
            timestamp: None,
            duration_ms: 0,
        });
    }

    records
}

fn parse_fish_history(content: &str) -> Vec<ImportRecord> {
    let mut records = Vec::new();

    let mut pending_command: Option<String> = None;
    let mut pending_timestamp: Option<DateTime<Utc>> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if let Some(cmd) = trimmed.strip_prefix("- cmd: ") {
            if let Some(prev_cmd) = pending_command.take() {
                records.push(ImportRecord {
                    command: prev_cmd,
                    timestamp: pending_timestamp.take(),
                    duration_ms: 0,
                });
            }

            pending_command = Some(cmd.to_string());
            pending_timestamp = None;
            continue;
        }

        if let Some(when_value) = trimmed.strip_prefix("when: ") {
            pending_timestamp = when_value
                .parse::<i64>()
                .ok()
                .and_then(parse_unix_timestamp);
        }
    }

    if let Some(prev_cmd) = pending_command.take() {
        records.push(ImportRecord {
            command: prev_cmd,
            timestamp: pending_timestamp,
            duration_ms: 0,
        });
    }

    records
}

pub fn default_history_path(shell: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("Could not determine home directory"))?;

    let path = match shell {
        "bash" => home.join(".bash_history"),
        "zsh" => home.join(".zsh_history"),
        "fish" => home.join(".local/share/fish/fish_history"),
        _ => return Err(anyhow!("Unsupported shell: {shell}")),
    };

    Ok(path)
}

pub fn import_history_file(
    conn: &Connection,
    shell: &str,
    file: Option<PathBuf>,
    allow_sensitive: bool,
) -> Result<ImportStats> {
    let kind = ShellKind::from_str(shell)?;

    let source = match file {
        Some(path) => path,
        None => default_history_path(shell)?,
    };

    let content = fs::read_to_string(&source)?;

    let parsed = match kind {
        ShellKind::Bash => parse_bash_history(&content),
        ShellKind::Zsh => parse_zsh_history(&content),
        ShellKind::Fish => parse_fish_history(&content),
    };

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

            let entry = HistoryEntry {
                id: None,
                command,
                cwd: "imported".to_string(),
                exit_code: 0,
                duration_ms: record.duration_ms,
                timestamp: record.timestamp.unwrap_or_else(Utc::now),
            };

            if db::history_entry_exists(conn, &entry)? {
                stats.skipped_duplicate += 1;
                continue;
            }

            db::insert_history_entry(conn, &entry)?;
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
