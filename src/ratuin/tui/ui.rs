use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use super::state::AppState;

pub fn draw(frame: &mut Frame<'_>, state: &AppState) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(frame.area());

    let query = Paragraph::new(state.query.as_str())
        .block(Block::default().borders(Borders::ALL).title("Query"));
    frame.render_widget(query, layout[0]);

    let items: Vec<ListItem<'_>> = state
        .entries
        .iter()
        .map(|entry| {
            let line = format!("{}  [{}]", entry.command, entry.cwd);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("History"))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    let mut list_state = ListState::default();
    if !state.entries.is_empty() {
        list_state.select(Some(state.selected));
    }
    frame.render_stateful_widget(list, layout[1], &mut list_state);

    let footer_text = format!(
        "{} | Enter: submit  Esc/Ctrl-C: quit  ↑/↓: navigate  Typing: search",
        state.status
    );
    let footer = Paragraph::new(footer_text).style(Style::default().fg(Color::Gray));
    frame.render_widget(footer, layout[2]);
}
