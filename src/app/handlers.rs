use anyhow::Result;
use chrono::Utc;

use crate::{db, importer, model::HistoryEntry, privacy, tui};

fn get_cwd(cwd: Option<String>) -> String {
    if let Some(cwd) = cwd
        && !cwd.is_empty()
    {
        cwd
    } else {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

pub fn handle_init(shell: String) -> Result<()> {
    let script = match shell.as_str() {
        "bash" => std::include_str!("../scripts/init.bash"),
        "zsh" => std::include_str!("../scripts/init.zsh"),
        "fish" => std::include_str!("../scripts/init.fish"),
        _ => anyhow::bail!("Unsupported shell: {}", shell),
    };

    println!("{}", script);
    Ok(())
}

pub fn handle_import(
    conn: &rusqlite::Connection,
    shell: String,
    file: Option<std::path::PathBuf>,
    allow_sensitive: bool,
) -> Result<()> {
    let stats = importer::import_history_file(conn, &shell, file, allow_sensitive)?;

    println!(
        "Imported {} entries from {} (skipped empty: {}, skipped sensitive: {}, skipped duplicate: {})",
        stats.imported,
        stats.source.display(),
        stats.skipped_empty,
        stats.skipped_sensitive,
        stats.skipped_duplicate,
    );

    Ok(())
}

pub fn handle_record(
    conn: &rusqlite::Connection,
    command: String,
    cwd: Option<String>,
    exit_code: u32,
    duration_ms: u64,
    allow_sensitive: bool,
) -> Result<()> {
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

    db::insert_history_entry(conn, &history_entry)?;
    println!("Recorded command: {}", history_entry);
    Ok(())
}

pub fn handle_search(
    conn: &rusqlite::Connection,
    keyword: String,
    limit: Option<usize>,
    cwd: Option<String>,
    failed_only: bool,
) -> Result<()> {
    let limit = limit.or(Some(50));
    let cwd_filter = cwd
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let results = db::search_history(conn, &keyword, limit, failed_only, cwd_filter)?;

    if results.is_empty() {
        println!("No results found for keyword: {}", keyword);
    } else {
        for entry in results {
            println!("{}", entry);
        }
    }

    Ok(())
}

pub fn handle_tui(
    keyword: Option<String>,
    cwd: Option<String>,
    limit: Option<usize>,
    failed_only: bool,
) -> Result<()> {
    let request = tui::TuiRequest {
        keyword,
        cwd,
        limit,
        failed_only,
    };

    tui::run(request)
}
