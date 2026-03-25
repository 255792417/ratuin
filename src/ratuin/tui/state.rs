use anyhow::Result;
use rusqlite::Connection;

use crate::ratuin::{db, model::HistoryEntry};

#[derive(Debug, Clone)]
pub struct TuiRequest {
    pub keyword: Option<String>,
    pub cwd: Option<String>,
    pub limit: Option<usize>,
    pub failed_only: bool,
    pub reverse_mode: bool,
}

pub struct AppState {
    pub query: String,
    pub cwd: Option<String>,
    pub limit: Option<usize>,
    pub failed_only: bool,
    pub entries: Vec<HistoryEntry>,
    pub selected: usize,
    pub should_exit: bool,
    pub selected_command: Option<String>,
    pub status: String,
    pub reverse_mode: bool,
}

impl AppState {
    pub fn from_request(request: TuiRequest) -> Self {
        let cwd = request
            .cwd
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);

        Self {
            query: request.keyword.unwrap_or_default(),
            cwd,
            limit: request.limit.or(Some(50)),
            failed_only: request.failed_only,
            entries: Vec::new(),
            selected: 0,
            should_exit: false,
            selected_command: None,
            status: String::new(),
            reverse_mode: request.reverse_mode,
        }
    }

    pub fn effective_query(&self) -> String {
        if self.reverse_mode {
            self.query.chars().rev().collect()
        } else {
            self.query.clone()
        }
    }

    pub fn render_query(&self) -> String {
        if self.reverse_mode {
            self.query.chars().rev().collect()
        } else {
            self.query.clone()
        }
    }

    pub fn refresh(&mut self, conn: &Connection) -> Result<()> {
        let effective_query = self.effective_query();
        self.entries = db::search_history(
            conn,
            &effective_query,
            self.limit,
            self.failed_only,
            self.cwd.as_deref(),
        )?;

        if self.reverse_mode {
            self.entries.reverse();
        }

        if self.entries.is_empty() {
            self.selected = 0;
            self.status = "No results".to_string();
        } else {
            if self.selected >= self.entries.len() {
                self.selected = self.entries.len() - 1;
            }
            self.status = format!("{} results", self.entries.len());
        }

        Ok(())
    }

    pub fn select_prev(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        if self.selected == 0 {
            self.selected = self.entries.len() - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn select_next(&mut self) {
        if self.entries.is_empty() {
            return;
        }

        self.selected = (self.selected + 1) % self.entries.len();
    }

    pub fn submit_selected(&mut self) {
        if let Some(entry) = self.entries.get(self.selected) {
            self.selected_command = Some(entry.command.clone());
        }
        self.should_exit = true;
    }
}
