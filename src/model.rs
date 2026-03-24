use std::fmt::{Display, Formatter};

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

impl HistoryEntry {
    fn fommated_time(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    fn time_passed(&self) -> String {
        let time_passed_secs = Utc::now()
            .signed_duration_since(self.timestamp)
            .num_seconds()
            .max(0);

        if time_passed_secs < 60 {
            format!("{}s ago", time_passed_secs)
        } else if time_passed_secs < 3600 {
            format!("{}m ago", time_passed_secs / 60)
        } else if time_passed_secs < 86400 {
            format!("{}h ago", time_passed_secs / 3600)
        } else {
            format!("{}d ago", time_passed_secs / 86400)
        }
    }

    fn formatted_duration(&self) -> String {
        if self.duration_ms < 1000 {
            format!("{}ms", self.duration_ms)
        } else {
            format!("{:.2}s", self.duration_ms as f64 / 1000.0)
        }
    }

    fn display_id(&self) -> String {
        match self.id {
            Some(id) => format!("[{}] ", id),
            None => "None ".to_string(),
        }
    }
}

impl Display for HistoryEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let id_part = self.display_id();
        let indent = " ".repeat(id_part.len());

        write!(
            f,
            "{id_part}{}\n\
    {indent}cwd: {}\n\
    {indent}exit code: {}, duration: {}\n\
    {indent}time: {}\n\
    {indent}executed: {}",
            self.command,
            self.cwd,
            self.exit_code,
            self.formatted_duration(),
            self.fommated_time(),
            self.time_passed()
        )?;

        Ok(())
    }
}
