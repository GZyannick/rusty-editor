use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode},
    style::{Color, Print, PrintStyledContent, Stylize},
    terminal::{self, size},
    ExecutableCommand, QueueableCommand,
};
use std::io::{Stdout, Write};

//How i will handle printing file
// I dont know if i print it char by char or word by word
// once printed didnt print it each time
// reprint it just if a change like save appens or compilation message for lsp

use crate::mode::Mode;
use crate::{action::Action, colors, buffer::Buffer};

pub struct Editor {
    pub mode: Mode,
    pub command: String,
    pub stdout: Stdout,
    pub size: (u16, u16),
    pub cursor: (u16, u16),
    pub buffer: Buffer,
}

impl Editor {
    pub fn new(buffer: Buffer) -> Result<Editor> {
        Ok(Editor {
            mode: Mode::Normal,
            command: String::new(),
            stdout: std::io::stdout(),
            size: size()?,
            cursor: (0, 0),
            buffer,
        })
    }

    pub fn enter_raw_mode(&mut self) -> anyhow::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        self.stdout
            .execute(crossterm::style::SetBackgroundColor(colors::BG_0))?;
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
                        self.buffer.add_char(c, &self.cursor);
                        self.cursor.0 += 1;
                    }
                    Action::RemoveChar => {
                        if self.cursor.0 > 0 {
                            self.buffer.remove_char(&self.cursor);
                            self.cursor.0 -= 1;
                        }
                        // TODO  add else remove char from line up
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
        self.draw_buffer()?;
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
            let nav = self.navigation(&code)?;
            if nav.is_some(){
                return Ok(nav);
            }

            return match self.mode {
                Mode::Normal => self.handle_normal_event(&code),
                Mode::Command => self.handle_command_event(&code),
                Mode::Insert => self.handle_insert_event(&code),
            };

        }
        Ok(None)
    }

    fn handle_insert_event(&mut self, code: &KeyCode) -> Result<Option<Action>> {
        match code {
            KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
            KeyCode::Backspace => Ok(Some(Action::RemoveChar)),
            KeyCode::Char(c) => Ok(Some(Action::AddChar(*c))),
            _ => Ok(None),
        }
    }

    fn handle_normal_event(&mut self, code: &KeyCode) -> Result<Option<Action>> {
        match code {
            KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),
            KeyCode::Char(':') => Ok(Some(Action::EnterMode(Mode::Command))),
            _ => Ok(None),
        }
    }

    fn handle_command_event(&mut self, code: &KeyCode) -> Result<Option<Action>> {
        match code {
            KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
            KeyCode::Char('q')  => Ok(Some(Action::Quit)),
            KeyCode::Char(c) => Ok(Some(Action::AddCommandChar(*c))),
            _ => Ok(None),
        }
    }

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

    fn draw_buffer(&mut self) -> Result<()> {
        self.stdout.queue(cursor::MoveTo(0, 0))?;

        for (i, line) in self.buffer.lines.iter().enumerate() {
            self.stdout
                .queue(PrintStyledContent(line.clone().on(colors::BG_0)))?;
            self.stdout.queue(cursor::MoveTo(0, i as u16))?;
        }

        Ok(())
    }

    fn draw_bottom_line(&mut self) -> Result<()> {
        // TODO find a separator

        self.stdout.queue(cursor::MoveTo(0, self.size.1 - 2))?;

        let mode = format!(" {} ", self.mode);
        let pos = format!(" {}:{} ", self.cursor.0, self.cursor.1);
        let filename = format!(" {}", self.buffer.path);
        let pad_width = self.size.0 - mode.len() as u16 - pos.len() as u16 - 2;
        let filename = format!(" {:<width$} ", filename, width = pad_width as usize);

        //print the mode
        self.stdout.queue(PrintStyledContent(
            mode.with(Color::White).bold().on(colors::STATUS_BG),
        ))?;

        //print the filename
        self.stdout.queue(PrintStyledContent(
            filename.with(Color::White).on(colors::FILE_STATUS_BG),
        ))?;

        // print the cursor position
        self.stdout.queue(PrintStyledContent(
            pos.with(Color::White).on(colors::STATUS_BG),
        ))?;

        self.stdout.flush()?;
        self.draw_command_line()?;

        Ok(())
    }

    fn draw_command_line(&mut self) -> Result<()> {
        if !self.command.is_empty() {
            self.stdout.queue(cursor::MoveTo(0, self.size.1 - 1))?;
            //TODO See if clear currentLine is the best for the text to not show behind the command
            self.stdout
                .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
            self.stdout.queue(Print(format!(":{}", self.command)))?;
        } else {
            self.stdout.queue(cursor::MoveTo(0, self.size.1 - 1))?;
            //TODO See if clear currentLine is the best for the text to not show behind the command
            self.stdout
                .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
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
