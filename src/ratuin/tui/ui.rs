use chrono::Utc;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use crate::ratuin::search;

use super::state::AppState;

fn relative_time_text(timestamp: chrono::DateTime<Utc>) -> String {
    let seconds = Utc::now()
        .signed_duration_since(timestamp)
        .num_seconds()
        .max(0);

    if seconds < 60 {
        format!("{}s ago", seconds)
    } else if seconds < 3600 {
        format!("{}m ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h ago", seconds / 3600)
    } else {
        format!("{}d ago", seconds / 86400)
    }
}

pub fn draw(frame: &mut Frame<'_>, state: &AppState) {
    let root_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let (results_area, query_area, footer_area) = if state.reverse_mode {
        (root_layout[0], root_layout[1], root_layout[2])
    } else {
        (root_layout[1], root_layout[0], root_layout[2])
    };

    let rendered_query = state.render_query();
    let query =
        Paragraph::new(rendered_query).block(Block::default().borders(Borders::ALL).title("Query"));
    frame.render_widget(query, query_area);

    let effective_query = state.effective_query();

    let items: Vec<ListItem<'_>> = state
        .entries
        .iter()
        .enumerate()
        .map(|(row_index, entry)| {
            let executed_at = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
            let relative = relative_time_text(entry.timestamp);
            let is_selected = row_index == state.selected;

            let selected_text_style = Style::default().fg(Color::Cyan);

            let match_indices =
                search::fuzzy_match_indices(&effective_query, &entry.command).unwrap_or_default();
            let mut command_spans = Vec::new();

            for (index, ch) in entry.command.chars().enumerate() {
                if match_indices.contains(&index) {
                    command_spans.push(Span::styled(
                        ch.to_string(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else if is_selected {
                    command_spans.push(Span::styled(ch.to_string(), selected_text_style));
                } else {
                    command_spans.push(Span::raw(ch.to_string()));
                }
            }

            let meta_line = if is_selected {
                Line::styled(
                    format!(
                        "  cwd: {}  |  time: {} ({})",
                        entry.cwd, executed_at, relative
                    ),
                    selected_text_style,
                )
            } else {
                Line::from(format!(
                    "  cwd: {}  |  time: {} ({})",
                    entry.cwd, executed_at, relative
                ))
            };

            ListItem::new(vec![Line::from(command_spans), meta_line])
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("History"))
        .highlight_style(Style::default())
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    if !state.entries.is_empty() {
        list_state.select(Some(state.selected));
    }
    frame.render_stateful_widget(list, results_area, &mut list_state);

    let footer_text = format!(
        "{} | mode: {} | Enter: execute  Tab: paste  Esc/Ctrl-C: quit  ↑/↓: navigate  Typing: search",
        state.status,
        if state.reverse_mode {
            "reverse"
        } else {
            "forward"
        }
    );
    let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::Gray));
    frame.render_widget(footer, footer_area);
}
