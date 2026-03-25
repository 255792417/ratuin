use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::process::Command;
use std::{
    io::{self, IsTerminal, Write},
    time::Duration,
};

use crate::ratuin::db;

mod state;
mod ui;

pub use state::TuiRequest;

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn shell_path() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
}

fn execute_selected_command(command: &str) -> Result<()> {
    let status = Command::new(shell_path())
        .arg("-ic")
        .arg(command)
        .status()?;

    if !status.success() {
        anyhow::bail!("Command exited with status: {}", status);
    }

    Ok(())
}

fn print_selected_command(command: &str, execute: bool) -> Result<()> {
    let mut stdout = io::stdout();

    if stdout.is_terminal() {
        if execute {
            writeln!(stdout, "{}", command)?;
        } else {
            write!(stdout, "{}", command)?;
        }
        stdout.flush()?;
    } else {
        println!("{}", command);
    }

    Ok(())
}

pub fn run(request: TuiRequest) -> Result<()> {
    let conn = db::open_db()?;
    let mut app = state::AppState::from_request(request);
    app.refresh(&conn)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut run_result: Result<()> = Ok(());

    while !app.should_exit {
        if let Err(error) = terminal.draw(|frame| ui::draw(frame, &app)) {
            run_result = Err(error.into());
            break;
        }

        match event::poll(Duration::from_millis(200)) {
            Ok(true) => match event::read() {
                Ok(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                    match (key.code, key.modifiers) {
                        (KeyCode::Esc, _) => app.should_exit = true,
                        (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.should_exit = true,
                        (KeyCode::Enter, _) => app.submit_selected(true),
                        (KeyCode::Tab, _) => app.submit_selected(false),
                        (KeyCode::Up, _) => app.select_prev(),
                        (KeyCode::Down, _) => app.select_next(),
                        (KeyCode::Backspace, _) => {
                            app.query.pop();
                            if let Err(error) = app.refresh(&conn) {
                                run_result = Err(error);
                                app.should_exit = true;
                            }
                        }
                        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
                            app.query.clear();
                            if let Err(error) = app.refresh(&conn) {
                                run_result = Err(error);
                                app.should_exit = true;
                            }
                        }
                        (KeyCode::Char(ch), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                            app.query.push(ch);
                            if let Err(error) = app.refresh(&conn) {
                                run_result = Err(error);
                                app.should_exit = true;
                            }
                        }
                        _ => {}
                    }
                }
                Ok(_) => {}
                Err(error) => {
                    run_result = Err(error.into());
                    break;
                }
            },
            Ok(false) => {}
            Err(error) => {
                run_result = Err(error.into());
                break;
            }
        }
    }

    let restore_result = restore_terminal(&mut terminal);
    if let Err(error) = restore_result {
        return Err(error);
    }

    run_result?;

    if let Some(command) = app.selected_command {
        if app.execute_selected {
            execute_selected_command(&command)?;
        } else {
            print_selected_command(&command, false)?;
        }
    }

    Ok(())
}
