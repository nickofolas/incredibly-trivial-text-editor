use crossterm::{
    event::{self, Event, KeyCode},
    terminal,
};
use std::{
    fs::OpenOptions,
    io::{self, prelude::*},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

enum StateMode {
    View,
    Edit,
    Command,
}

struct State {
    mode: StateMode,
    content_body: String,
    content_command: String,
}

fn main() -> Result<()> {
    let stdout = io::stdout();
    terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut term = Terminal::new(backend)?;

    let mut state = State {
        mode: StateMode::Edit,
        content_body: if std::env::args().count() > 1 {
            let mut file = OpenOptions::new()
                .read(true)
                .create(true)
                .append(true)
                .write(true)
                .open(std::env::args().last().unwrap())?;
            let mut buffer = String::new();
            file.read_to_string(&mut buffer)?;
            buffer
        } else {
            String::new()
        },
        content_command: String::new(),
    };

    term.clear()?;
    loop {
        term.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(4), Constraint::Length(3)].as_ref())
                .split(f.size());

            match state.mode {
                StateMode::Edit => {
                    state.content_command = String::from("Mode: EDIT");
                    let split_body = state.content_body.split('\n');
                    f.set_cursor(
                        chunks[0].x + split_body.clone().last().unwrap().len() as u16 + 1,
                        chunks[0].y + split_body.count() as u16,
                    );
                }
                StateMode::Command => {
                    f.set_cursor(
                        chunks[1].x + state.content_command.len() as u16 + 1,
                        chunks[1].y + 1,
                    );
                }
                StateMode::View => state.content_command = String::from("Mode: VIEW"),
            }

            let body = Paragraph::new(state.content_body.as_ref())
                .block(Block::default().borders(Borders::ALL));
            let command_bar = Paragraph::new(state.content_command.as_ref())
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(body, chunks[0]);
            f.render_widget(command_bar, chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match state.mode {
                StateMode::View => match key.code {
                    KeyCode::Char(':') => {
                        state.mode = StateMode::Command;
                        state.content_command.clear();
                        state.content_command.push(':');
                    }
                    KeyCode::Char('i') => state.mode = StateMode::Edit,
                    _ => {}
                },

                StateMode::Edit => match key.code {
                    KeyCode::Enter => state.content_body.push('\n'),
                    KeyCode::Esc => state.mode = StateMode::View,
                    KeyCode::Tab => state.content_body.push_str("    "),
                    KeyCode::Backspace => {
                        state.content_body.pop();
                    }
                    KeyCode::Char(c) => state.content_body.push(c),
                    _ => {}
                },

                StateMode::Command => match key.code {
                    KeyCode::Enter => {
                        let content = state.content_command.clone();
                        let command = content.strip_prefix(":").unwrap();
                        state.content_command.clear();
                        match String::from(command)
                            .clone()
                            .split_ascii_whitespace()
                            .collect::<Vec<&str>>()[0]
                        {
                            "quit" | "q" => break,
                            "write" | "w" => {
                                let filename = command.split_ascii_whitespace().last().unwrap();
                                let mut file =
                                    OpenOptions::new().write(true).create(true).open(filename)?;
                                file.write_all(state.content_body.clone().as_bytes())?;
                            }
                            _ => {}
                        }
                    }
                    KeyCode::Esc => {
                        state.content_command.clear();
                        state.mode = StateMode::View;
                    }
                    KeyCode::Backspace => {
                        state.content_command.pop();
                    }
                    KeyCode::Char(c) => state.content_command.push(c),
                    _ => {}
                },
            }
        };
    }

    terminal::disable_raw_mode()?;
    term.set_cursor(0, 0)?;
    term.clear()?;
    Ok(())
}
