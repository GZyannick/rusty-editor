use anyhow::Result;
use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode},
    style::{Color, Print, PrintStyledContent, Stylize},
    terminal::{self, size},
    ExecutableCommand, QueueableCommand,
};
use std::{
    io::{Stdout, Write},
    u16, usize,
};

use crate::mode::Mode;
use crate::{action::Action, buffer::Buffer, colors, viewport::Viewport};

pub const TERMINAL_SIZE_MINUS: u16 = 2;

pub struct Editor {
    pub mode: Mode,
    pub command: String,
    pub stdout: Stdout,
    pub size: (u16, u16),
    pub cursor: (u16, u16),
    pub viewport: Viewport,
}

impl Editor {
    pub fn new(buffer: Buffer) -> Result<Editor> {
        let size = size()?;
        let viewport = Viewport::new(buffer, size.0, size.1);

        Ok(Editor {
            mode: Mode::Normal,
            command: String::new(),
            stdout: std::io::stdout(),
            size,
            cursor: (0, 0),
            viewport,
        })
    }

    pub fn enter_raw_mode(&mut self) -> anyhow::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        self.stdout
            .execute(crossterm::style::SetBackgroundColor(colors::BG_0))?;
        self.stdout.execute(terminal::EnterAlternateScreen)?;
        self.stdout
            .execute(terminal::SetSize(self.size.0 - TERMINAL_SIZE_MINUS, self.size.1 - TERMINAL_SIZE_MINUS))?;
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
                    Action::MoveUp if self.cursor.1 == 0 && self.viewport.y_pos > 0 => {
                        self.viewport.scroll_up();
                    }
                    Action::MoveLeft if self.cursor.0 > 0 => {
                        self.cursor.0 -= 1;
                    }
                    Action::MoveRight => {
                        self.cursor.0 += 1;
                    }
                    Action::MoveDown if self.max_cursor_viewport_height() => {
                        let cursor_viewport =
                            self.viewport.get_cursor_viewport_position(&self.cursor);
                        if (cursor_viewport.1 as usize) < (self.viewport.buffer.lines.len() - TERMINAL_SIZE_MINUS as usize) {
                            self.viewport.scroll_down();
                        }
                    }
                    Action::MoveDown => {
                        // TODO stop cursor at the end of the file
                        let cursor_viewport =
                            self.viewport.get_cursor_viewport_position(&self.cursor);
                        if (cursor_viewport.1 as usize) < (self.viewport.buffer.lines.len() - TERMINAL_SIZE_MINUS as usize) {
                            self.cursor.1 += 1;
                        }
                    }
                    Action::AddChar(c) => {
                        self.cursor.0 += 1;
                        let cursor_viewport =
                            self.viewport.get_cursor_viewport_position(&self.cursor);
                        self.viewport.buffer.add_char(c, cursor_viewport);
                    }
                    Action::RemoveChar => {
                        if self.cursor.0 > 0 {
                            let cursor_viewport =
                                self.viewport.get_cursor_viewport_position(&self.cursor);
                            self.viewport.buffer.remove_char(cursor_viewport);
                            self.cursor.0 -= 1;
                        }
                        // TODO  add else remove char from line up above
                    }
                    Action::EnterMode(mode) => {
                        self.mode = mode;
                    }
                    Action::AddCommandChar(c) => {
                        self.command.push(c);
                    }
                    Action::Resize => {
                        let size = size()?;
                        self.viewport.width = size.0;
                        self.viewport.height = size.1;
                        self.size = size;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn max_cursor_viewport_height(&self) -> bool {
        // the status line and command line take two line and we dont want the cursor to go on them
        // so max_cursor_height is -3 the height
        // on the mode line so -3
       self.cursor.1 >= (self.viewport.height - (TERMINAL_SIZE_MINUS + 1_u16))
    }


    fn handle_action(&mut self, event: Event) -> Result<Option<Action>> {
        if let event::Event::Key(ev) = event {
            if ev.kind == event::KeyEventKind::Release {
                return Ok(None);
            }

            let code = ev.code;
            let nav = self.navigation(&code)?;
            if nav.is_some() {
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
            KeyCode::Char('q') => Ok(Some(Action::Quit)),
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

    pub fn draw(&mut self) -> Result<()> {
        self.draw_buffer()?;
        self.draw_bottom_line()?;
        self.stdout
            .queue(cursor::MoveTo(self.cursor.0, self.cursor.1))?;
        self.stdout.flush()?;
        Ok(())
    }



    fn draw_buffer(&mut self) -> Result<()> {
        self.stdout.queue(cursor::MoveTo(0, 0))?;

        // TODO see if clearAll is the better option to remove undesired artifact when a char is
        // remove
        //TODO see how i can reprint the bg if i still do ClearType::All
        //self.stdout.queue(crossterm::style::SetBackgroundColor(colors::BG_0))?;
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?;

        for (i, line) in self.viewport.get_buffer_viewport().iter().enumerate() {
            self.stdout
                .queue(PrintStyledContent(line.clone().on(colors::BG_0)))?;
            self.stdout.queue(cursor::MoveTo(0, i as u16))?;
        }

        Ok(())
    }


    fn draw_bottom_line(&mut self) -> Result<()> {
        // TODO find a separator

        self.stdout.queue(cursor::MoveTo(0, self.size.1 - TERMINAL_SIZE_MINUS ))?;

        let mode = format!(" {} ", self.mode);
        let pos = format!(" {}:{} ", self.cursor.0, self.cursor.1);
        let filename = format!(" {}", self.viewport.buffer.path);
        let pad_width = self.size.0 - mode.len() as u16 - pos.len() as u16 - TERMINAL_SIZE_MINUS;
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
            self.stdout.queue(cursor::MoveTo(0, self.size.1 - 1 ))?;
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
