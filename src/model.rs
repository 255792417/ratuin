use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct HistoryEntry {
    pub id: Option<i64>,
    pub command: String,
    pub cwd: String,
    pub exit_code: i32,
    pub duration_ms: i64,
    pub timestamp: DateTime<Utc>,
}
