use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode},
    style::{Attribute, Color, Print, PrintStyledContent, Stylize},
    terminal::{self, size},
    ExecutableCommand, QueueableCommand,
};
use std::io::{Stdout, Write};

//How i will handle printing file
// I dont know if i print it char by char or word by word
// once printed didnt print it each time
// reprint it just if a change like save appens or compilation message for lsp

use crate::{action::Action, colors};
use crate::mode::Mode;

pub struct Editor {
    pub mode: Mode,
    pub command: String,
    pub stdout: Stdout,
    pub size: (u16, u16),
    pub cursor: (u16, u16),
    //pub file_buffer: Vec<u8>,
}

impl Editor {
    pub fn new() -> Result<Editor> {
        //Test purpose
        //let mut file_buffer: Vec<u8> = vec![];
        //let mut file = std::fs::File::open("./src/main.rs")?;
        //file.read_to_end(&mut file_buffer)?;
        //
        //
        //Test purpose

        Ok(Editor {
            mode: Mode::Normal,
            command: String::new(),
            stdout: std::io::stdout(),
            size: size()?,
            cursor: (0, 0),
            //file_buffer,
        })
    }

    pub fn enter_raw_mode(&mut self) -> anyhow::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        self.stdout.execute(crossterm::style::SetBackgroundColor(colors::BG_0))?;
        self.stdout.execute(terminal::EnterAlternateScreen)?;
        self.stdout
            .execute(terminal::SetSize(self.size.0, self.size.1 - 2))?;
        self.stdout
            .execute(terminal::Clear(terminal::ClearType::All))?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.draw()?;
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

    pub fn draw(&mut self) -> Result<()> {
        self.draw_bottom_line()?;
        self.stdout
            .queue(cursor::MoveTo(self.cursor.0, self.cursor.1))?;
        self.stdout.flush()?;
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

    // TODO replace handle action in each specific fn of mode
    fn handle_insert_event(&mut self, code: KeyCode) -> Result<()> {todo!()}
    fn handle_normal_event(&mut self, code: KeyCode) -> Result<()> {todo!()}
    fn handle_command_event(&mut self, code: KeyCode) -> Result<()> {todo!()}

    fn navigation(&mut self, code: &KeyCode) -> Result<Option<Action>> {
        let mut action: Option<Action> = None;

        if matches!(self.mode, Mode::Command) {
            return Ok(action);
        }

        action = match code {
            KeyCode::Down => Some(Action::MoveDown),
            KeyCode::Up => Some(Action::MoveUp),
            KeyCode::Left => Some(Action::MoveLeft),
            KeyCode::Right => Some(Action::MoveRight),
            _ => None,
        };

        if !matches!(self.mode, Mode::Insert) && action.is_none() {
            action = match code {
                KeyCode::Char('h') => Some(Action::MoveLeft),
                KeyCode::Char('j') => Some(Action::MoveDown),
                KeyCode::Char('k') => Some(Action::MoveUp),
                KeyCode::Char('l') => Some(Action::MoveRight),
                _ => None,
            }
        };

        Ok(action)
    }

    fn draw_bottom_line(&mut self) -> Result<()> {
        self.stdout.queue(cursor::MoveTo(0, self.size.1 - 2))?;
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;

        let mode_style = PrintStyledContent(
            format!(" {} ", self.mode)
                .with(Color::White).attribute(Attribute::Bold)
                .on(colors::STATUS_BG)
        );

        self.stdout.queue(mode_style)?;
        // TODO find a separator
        //self.stdout.queue(PrintStyledContent("".with(colors::STATUS_BG).bold()))?;

        //TODO ADD real file name
        self.stdout.queue(PrintStyledContent(" /src/placeholder.rs ".with(Color::White).bold().on(colors::FILE_STATUS_BG)))?;

        self.stdout.flush()?;
        self.draw_command_line()?;

        Ok(())
    }

    fn draw_command_line(&mut self) -> Result<()> {
        if !self.command.is_empty() {
            self.stdout.queue(cursor::MoveTo(0, self.size.1 - 1))?;
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
