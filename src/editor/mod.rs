mod ui;
use ui::Draw;
mod core;
use crate::buff::Buffer;
use crate::theme::colors;
use crate::viewport::Viewport;
use core::{action::Action, mode::Mode};

use anyhow::{Ok, Result};
use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode},
    style::{Color, Print, PrintStyledContent, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::{Stdout, Write};

pub const TERMINAL_SIZE_MINUS: u16 = 2;

#[derive(Debug)]
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
        let size = terminal::size()?;
        let viewport = Viewport::new(buffer, size.0 - 3, size.1 - TERMINAL_SIZE_MINUS);

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
            .execute(terminal::Clear(terminal::ClearType::All))?;
        self.stdout
            .execute(terminal::SetSize(self.size.0, self.size.1))?;

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.draw()?;
            let event = read()?;

            if let event::Event::Resize(width, height) = event {
                self.viewport.width = width;
                self.viewport.height = height;
                self.size = (width, height);
                self.stdout
                    .queue(terminal::Clear(terminal::ClearType::All))?;
                continue;
            }

            if let Some(action) = self.handle_action(event)? {
                match action {
                    Action::Quit => break,
                    Action::MoveUp => {
                        self.move_prev_line();
                        self.cursor.0 = self.viewport.get_cursor_max_x_position(&self.cursor);
                    }

                    Action::MoveRight => {
                        if self.viewport.get_line_len(&self.cursor) > self.cursor.0 {
                            self.cursor.0 += 1;
                        } else {
                            // if we are at the end of the line go ot the next line if exist
                            // and move the cursor to the start of the line
                            self.move_next_line(0);
                        }
                    }
                    Action::MoveLeft => {
                        if self.cursor.0 > 0 {
                            self.cursor.0 -= 1;
                        } else if self.cursor.0 == 0 && (self.cursor.1 > 0 || self.viewport.top > 0)
                        {
                            // if we are at the start of the line go ot the prev line if exist
                            // and move the cursor to the end of the line
                            self.move_prev_line();
                            self.cursor.0 = self.viewport.get_line_len(&self.cursor);
                        }
                    }

                    Action::MoveDown => {
                        self.move_next_line(self.viewport.get_cursor_max_x_position(&self.cursor));
                    }
                    Action::AddChar(c) => {
                        self.cursor.0 += 1;
                        let cursor_viewport =
                            self.viewport.get_cursor_viewport_position(&self.cursor);
                        self.viewport.buffer.add_char(c, cursor_viewport);
                    }
                    Action::RemoveChar => {
                        let cursor_viewport =
                            self.viewport.get_cursor_viewport_position(&self.cursor);
                        match cursor_viewport.0 > 0 {
                            true => {
                                self.viewport.buffer.remove_char(cursor_viewport);
                                self.cursor.0 -= 1;
                            }
                            false if cursor_viewport.1 > 0 => {
                                // we get the size of the prev line before change
                                // because we want the text that will be added behind the cursor
                                let new_x_pos = self
                                    .viewport
                                    .get_line_len(&(self.cursor.0, self.cursor.1 - 1));
                                self.viewport.buffer.remove_char_line(cursor_viewport);
                                self.move_prev_line();
                                self.cursor.0 = new_x_pos;
                            }
                            _ => {}
                        }
                    }
                    Action::EnterMode(mode) => {
                        self.mode = mode;
                    }
                    Action::AddCommandChar(c) => {
                        self.command.push(c);
                    }
                    Action::NewLine(is_enter) => {
                        let v_cursor = self.viewport.get_cursor_viewport_position(&self.cursor);
                        self.viewport.buffer.new_line(v_cursor, is_enter);
                        self.move_next_line(self.viewport.get_cursor_max_x_position(&self.cursor));
                        self.cursor.0 = 0;
                    }
                    Action::SaveFile => {
                        self.viewport.buffer.save()?;
                    } //_ => {}
                }
            }
        }
        Ok(())
    }

    fn move_prev_line(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else {
            self.viewport.scroll_up();
        }
    }

    fn move_next_line(&mut self, x_pos: u16) {
        if self.viewport.is_under_buffer_len(&self.cursor) {
            match self.max_cursor_viewport_height() {
                true => {
                    self.viewport.scroll_down();
                }
                false => {
                    self.cursor.1 += 1;
                }
            }

            self.cursor.0 = x_pos;
        }
    }

    fn max_cursor_viewport_height(&self) -> bool {
        self.cursor.1 >= (self.viewport.height - 1)
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
            KeyCode::Enter => Ok(Some(Action::NewLine(true))),
            KeyCode::Char(c) => Ok(Some(Action::AddChar(*c))),
            _ => Ok(None),
        }
    }

    fn handle_normal_event(&mut self, code: &KeyCode) -> Result<Option<Action>> {
        match code {
            KeyCode::Char('i') => Ok(Some(Action::EnterMode(Mode::Insert))),
            KeyCode::Char(':') => Ok(Some(Action::EnterMode(Mode::Command))),
            KeyCode::Char('o') => Ok(Some(Action::NewLine(false))),
            _ => Ok(None),
        }
    }

    fn handle_command_event(&mut self, code: &KeyCode) -> Result<Option<Action>> {
        match code {
            KeyCode::Esc => Ok(Some(Action::EnterMode(Mode::Normal))),
            KeyCode::Char('q') => Ok(Some(Action::Quit)),
            KeyCode::Char('w') => Ok(Some(Action::SaveFile)),
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
}

impl Draw for Editor {
    fn draw(&mut self) -> Result<()> {
        self.viewport.draw(&mut self.stdout)?;
        self.draw_bottom()?;
        self.stdout
            .queue(cursor::MoveTo(self.cursor.0, self.cursor.1))?;
        self.stdout.flush()?;
        Ok(())
    }

    fn draw_bottom(&mut self) -> anyhow::Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, self.size.1 - TERMINAL_SIZE_MINUS))?;

        let cursor_viewport = self.viewport.get_cursor_viewport_position(&self.cursor);

        let mode = format!(" {} ", self.mode);
        let pos = format!(" {}:{} ", cursor_viewport.0, cursor_viewport.1);
        let pad_width = self.size.0 - mode.len() as u16 - pos.len() as u16 - TERMINAL_SIZE_MINUS;
        let filename = format!(
            " {:<width$} ",
            self.viewport.buffer.path,
            width = pad_width as usize
        );

        self.draw_status_line(mode, filename)?;
        self.draw_line_counter(pos)?;
        self.draw_command_line()?;

        Ok(())
    }

    fn draw_status_line(&mut self, mode: String, filename: String) -> Result<()> {
        self.stdout.queue(PrintStyledContent(
            mode.with(Color::White).bold().on(colors::STATUS_BG),
        ))?;

        //print the filename
        self.stdout.queue(PrintStyledContent(
            filename.with(Color::White).on(colors::FILE_STATUS_BG),
        ))?;
        Ok(())
    }

    fn draw_line_counter(&mut self, pos: String) -> Result<()> {
        // print the cursor position
        self.stdout.queue(PrintStyledContent(
            pos.with(Color::White).on(colors::STATUS_BG),
        ))?;

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
        let _ = self
            .stdout
            .execute(terminal::Clear(terminal::ClearType::Purge));
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}
