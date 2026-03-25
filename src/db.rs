use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::fs;
use std::path::PathBuf;

use crate::model::HistoryEntry;
use crate::search::is_fuzzy_subsequence;

fn map_history_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<HistoryEntry> {
    Ok(HistoryEntry {
        id: row.get(0)?,
        command: row.get(1)?,
        cwd: row.get(2)?,
        exit_code: row.get(3)?,
        duration_ms: u64::try_from(row.get::<_, i64>(4)?).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Integer,
                Box::new(e),
            )
        })?,
        timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?
            .with_timezone(&Utc),
    })
}

fn build_search_query(failed_only: bool, has_cwd_filter: bool) -> String {
    let mut query = if failed_only {
        r#"
        SELECT id, command, cwd, exit_code, duration_ms, timestamp
        FROM history
        WHERE exit_code != 0
        "#
        .to_string()
    } else {
        r#"
        SELECT id, command, cwd, exit_code, duration_ms, timestamp
        FROM history
        WHERE 1 = 1
        "#
        .to_string()
    };

    if has_cwd_filter {
        query.push_str(" AND cwd = ?1");
    }

    query.push_str(" ORDER BY id DESC");
    query
}

fn push_if_fuzzy_match(
    entries: &mut Vec<HistoryEntry>,
    keyword: &str,
    entry: HistoryEntry,
    limit: Option<usize>,
) -> bool {
    if is_fuzzy_subsequence(keyword, &entry.command) {
        entries.push(entry);
        if let Some(max) = limit {
            return entries.len() >= max;
        }
    }

    false
}

pub fn data_dir() -> Result<PathBuf> {
    let data_dir =
        dirs::data_local_dir().ok_or_else(|| anyhow!("Could not determine data directory"))?;
    let ratuin_dir = data_dir.join("ratuin");

    if !ratuin_dir.exists() {
        fs::create_dir_all(&ratuin_dir)?;
    }

    Ok(ratuin_dir)
}

pub fn db_path() -> Result<PathBuf> {
    let ratuin_dir = data_dir()?;
    Ok(ratuin_dir.join("history.db"))
}

pub fn open_db() -> Result<Connection> {
    let db_path = db_path()?;
    let conn = Connection::open(db_path)?;

    init_db(&conn)?;
    Ok(conn)
}

fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            command TEXT NOT NULL,
            cwd TEXT NOT NULL,
            exit_code INTEGER NOT NULL,
            duration_ms INTEGER NOT NULL,
            timestamp TEXT NOT NULL
        );
        "#,
    )?;
    Ok(())
}

pub fn insert_history_entry(conn: &Connection, entry: &HistoryEntry) -> Result<()> {
    conn.execute(
        r#"INSERT INTO 
        history (command, cwd, exit_code, duration_ms, timestamp) 
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![
            &entry.command,
            &entry.cwd,
            entry.exit_code,
            i64::try_from(entry.duration_ms)
                .map_err(|e| anyhow::anyhow!("Failed to convert duration_ms: {}", e))?,
            entry.timestamp.to_rfc3339(),
        ],
    )?;
    Ok(())
}

pub fn history_entry_exists(conn: &Connection, entry: &HistoryEntry) -> Result<bool> {
    let mut stmt = conn.prepare(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM history
            WHERE command = ?1
              AND cwd = ?2
              AND timestamp = ?3
        )
        "#,
    )?;

    let exists: i64 = stmt.query_row(
        params![&entry.command, &entry.cwd, entry.timestamp.to_rfc3339()],
        |row| row.get(0),
    )?;

    Ok(exists != 0)
}

pub fn search_history(
    conn: &Connection,
    keyword: &str,
    limit: Option<usize>,
    failed_only: bool,
    cwd: Option<&str>,
) -> Result<Vec<HistoryEntry>> {
    let query = build_search_query(failed_only, cwd.is_some());
    let mut stmt = conn.prepare(&query)?;
    let mut rows = if let Some(cwd_value) = cwd {
        stmt.query(params![cwd_value])?
    } else {
        stmt.query([])?
    };

    let mut entries = Vec::new();
    while let Some(row) = rows.next()? {
        let entry = map_history_entry(row)?;
        if push_if_fuzzy_match(&mut entries, keyword, entry, limit) {
            break;
        }
    }

    Ok(entries)
}
