use std::io::{Stdout, Write};

use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode},
    style::{Attribute, Color, Print, PrintStyledContent, Stylize},
    terminal::{self, size},
    ExecutableCommand, QueueableCommand,
};

use crate::mode::Mode;
use crate::action::Action;


pub struct Editor {
    pub mode: Mode,
    pub command: String,
    pub stdout: Stdout,
    pub size: (u16, u16),
    pub cursor: (u16, u16),
}

impl Editor {
    pub fn new() -> Result<Editor> {
        Ok(Editor {
            mode: Mode::Normal,
            command: String::new(),
            stdout: std::io::stdout(),
            size: size()?,
            cursor: (0, 0),
        })
    }

    pub fn enter_raw_mode(&mut self) -> anyhow::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        self.stdout.execute(terminal::EnterAlternateScreen)?;
        self.stdout.execute(terminal::SetSize(self.size.0, self.size.1 - 2))?;
        self.stdout.execute(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        loop {
            self.draw_bottom_line()?;
            self.stdout
                .queue(cursor::MoveTo(self.cursor.0, self.cursor.1))?;
            self.stdout.flush()?;


            let event = read()?;
            if let Some(action) = self.handle_action(event)? {
                match action {
                    Action::Quit => break,
                    Action::MoveUp if self.cursor.1 > 0 => {
                        self.cursor.1 -= 1;
                    }
                    Action::MoveLeft if self.cursor.0 > 0 => {
                        self.cursor.0 -= 1;
                    }
                    Action::MoveRight => {
                        self.cursor.0 += 1;
                    }
                    Action::MoveDown => {
                        self.cursor.1 += 1;
                    }
                    Action::AddChar(c) => {
                        self.stdout.queue(Print(c.to_string()))?;
                        self.cursor.0 += 1;
                    }
                    Action::EnterMode(mode) => {
                        self.mode = mode;
                    }
                    Action::AddCommandChar(c) => {
                        self.command.push(c);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn handle_action(&mut self, event: Event) -> Result<Option<Action>> {
        if let event::Event::Key(ev) = event {
            if ev.kind == event::KeyEventKind::Release {
                return Ok(None);
            }

            let code = ev.code;
            if let Some(action) = self.navigation(&code)? {
                return Ok(Some(action));
            }

            return match code {
                KeyCode::Char('q') if matches!(self.mode, Mode::Command) => Ok(Some(Action::Quit)),
                KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),
                KeyCode::Char(':') => Ok(Some(Action::EnterMode(Mode::Command))),
                KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
                KeyCode::Char(c) if matches!(self.mode, Mode::Insert) => {
                    Ok(Some(Action::AddChar(c)))
                }
                KeyCode::Char(c) if matches!(self.mode, Mode::Command) => {
                    Ok(Some(Action::AddCommandChar(c)))
                }
                _ => Ok(None),
            };
        }
        Ok(None)
    }

    fn navigation(&mut self, code: &KeyCode) -> Result<Option<Action>> {
        if matches!(self.mode, Mode::Insert) | matches!(self.mode, Mode::Command) {
            return match code {
                KeyCode::Down => Ok(Some(Action::MoveDown)),
                KeyCode::Up => Ok(Some(Action::MoveUp)),
                KeyCode::Left => Ok(Some(Action::MoveLeft)),
                KeyCode::Right => Ok(Some(Action::MoveRight)),
                _ => Ok(None),
            };
        }
        match code {
            KeyCode::Down => Ok(Some(Action::MoveDown)),
            KeyCode::Up => Ok(Some(Action::MoveUp)),
            KeyCode::Left => Ok(Some(Action::MoveLeft)),
            KeyCode::Right => Ok(Some(Action::MoveRight)),
            KeyCode::Char('h') => Ok(Some(Action::MoveLeft)),
            KeyCode::Char('j') => Ok(Some(Action::MoveDown)),
            KeyCode::Char('k') => Ok(Some(Action::MoveUp)),
            KeyCode::Char('l') => Ok(Some(Action::MoveRight)),
            _ => Ok(None),
        }
    }

    fn draw_bottom_line(&mut self) -> Result<()> {
        self.stdout.queue(cursor::MoveTo(0, self.size.1 - 2))?;
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;

        let mode_style = PrintStyledContent(
            format!(" {} ", self.mode)
                .with(Color::White)
                .on(Color::Rgb {
                    r: 188,
                    g: 150,
                    b: 230,
                })
                .attribute(Attribute::Bold),
        );

        self.stdout.queue(mode_style)?;
        if !self.command.is_empty() {
            self.stdout.queue(cursor::MoveTo(0, self.cursor.1))?;
            self.stdout.queue(Print(format!(":{}", self.command)))?;
        }

        Ok(())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
