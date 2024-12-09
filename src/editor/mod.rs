mod ui;
use ui::Draw;
mod core;
use crate::buff::Buffer;
use crate::theme::colors;
use crate::viewport::Viewport;
use anyhow::{Ok, Result};
use core::{
    action::{self, Action},
    mode::Mode,
};
use crossterm::{
    cursor,
    event::{self, read, Event, KeyCode, KeyModifiers},
    style::{Color, Print, PrintStyledContent, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::{Stdout, Write};

// TERMINAL_LINE_LEN_MINUS if we want the cursor to go behind the last char or stop before,
// 1: stop on char, 0: stop after the char
pub const TERMINAL_LINE_LEN_MINUS: u16 = 1;
pub const TERMINAL_SIZE_MINUS: u16 = 2; // we remove the size of the bottom status, command bar
pub const MOVE_PREV_OR_NEXT_LINE: bool = true; // on true allow us to activate the feature where if we
                                               // are at the end of the line or start move to next or prev line

#[derive(Debug)]
// TODO: FIND THE BUG WHERE THE LAST LINE OF THE VIEWPORT BEING BAD DRAW
pub struct Editor {
    pub mode: Mode,
    pub command: String,
    pub stdout: Stdout,
    pub size: (u16, u16),
    pub cursor: (u16, u16),
    pub buffer_x_cursor: u16,
    pub waiting_command: Option<char>,
    pub viewport: Viewport,
    pub buffer_actions: Vec<Action>, // allow us to buffer some action to make multiple of them in one time
    pub undo_actions: Vec<Action>,   // create a undo buffer where we put all the action we want
    pub undo_insert_actions: Vec<Action>, // when we are in insert mode all the undo at the same
                                     // place
                                     // PS i could do better on comment
}

impl Editor {
    pub fn new(buffer: Buffer) -> Result<Editor> {
        let size = terminal::size()?;
        let viewport = Viewport::new(buffer, size.0, size.1 - TERMINAL_SIZE_MINUS);
        // let viewport = Viewport::new(buffer, size.0 - 3, size.1 - TERMINAL_SIZE_MINUS);

        Ok(Editor {
            mode: Mode::Normal,
            command: String::new(),
            stdout: std::io::stdout(),
            size,
            cursor: (0, 0),
            buffer_x_cursor: 0,
            waiting_command: None,
            viewport,
            buffer_actions: vec![],
            undo_actions: vec![],
            undo_insert_actions: vec![],
        })
    }

    pub fn v_cursor(&self) -> (u16, u16) {
        self.viewport.viewport_cursor(&self.cursor)
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

    fn clear_buffer_x_cursor(&mut self) {
        self.buffer_x_cursor = 0;
    }

    fn check_bounds(&mut self) {
        let line_len = self.get_specific_line_len_by_mode();

        if self.cursor.0 >= line_len {
            if self.buffer_x_cursor == 0 {
                self.buffer_x_cursor = self.cursor.0;
            }
            self.cursor.0 = line_len;
        } else if self.cursor.0 < line_len && self.buffer_x_cursor > 0 {
            // allow us to add a buffer to the cursor to return to its original position
            // when he move on multiple line that was inferior of the cursor.0
            match self.buffer_x_cursor < line_len {
                true => {
                    self.cursor.0 = self.buffer_x_cursor;
                    self.clear_buffer_x_cursor();
                }
                false => {
                    self.cursor.0 = line_len;
                }
            }
        }

        if self.v_cursor().1 as usize >= self.viewport.get_buffer_len() {
            self.cursor.1 -= 1;
        }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.check_bounds();
            self.draw()?;
            let event = read()?;

            if let event::Event::Resize(width, height) = event {
                self.resize(width, height)?;
                continue;
            }

            if let Some(action) = self.handle_action(event)? {
                if matches!(action, Action::Quit) {
                    break;
                }
                action.execute(self)?;
            }
        }
        Ok(())
    }

    fn resize(&mut self, w: u16, h: u16) -> Result<()> {
        self.viewport.vwidth = w;
        self.viewport.vheight = h;
        self.size = (w, h);
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?;

        Ok(())
    }

    fn move_prev_line(&mut self) {
        match self.cursor.1 > 0 {
            true => self.cursor.1 -= 1,
            false => self.viewport.scroll_up(),
        }
    }

    fn move_next_line(&mut self) {
        if self.viewport.is_under_buffer_len(&self.cursor) {
            match self.cursor.1 >= (self.viewport.vheight - 1) {
                true => self.viewport.scroll_down(),
                false => self.cursor.1 += 1,
            }
        }
    }

    fn handle_action(&mut self, event: Event) -> Result<Option<Action>> {
        if let event::Event::Key(ev) = event {
            if ev.kind == event::KeyEventKind::Release {
                return Ok(None);
            }

            let code = ev.code;
            let modifiers = ev.modifiers;

            if let Some(c) = self.waiting_command {
                let action = self.handle_waiting_command(c, &code);
                self.waiting_command = None;
                self.stdout
                    .queue(cursor::SetCursorStyle::DefaultUserShape)?;
                return Ok(action);
            }

            let nav = self.navigation(&code)?;
            if nav.is_some() {
                return Ok(nav);
            }

            return match self.mode {
                Mode::Normal => self.handle_normal_event(&code, &modifiers),
                Mode::Command => self.handle_command_event(&code, &modifiers),
                Mode::Insert => self.handle_insert_event(&code, &modifiers),
            };
        }
        Ok(None)
    }

    fn handle_waiting_command(&mut self, c: char, code: &KeyCode) -> Option<Action> {
        match c {
            'd' => match code {
                KeyCode::Char('d') => Some(Action::DeleteLine),
                KeyCode::Char('w') => Some(Action::DeleteWord),
                _ => None,
            },
            'g' => match code {
                KeyCode::Char('g') => Some(Action::StartOfFile),
                _ => None,
            },
            'z' => match code {
                KeyCode::Char('z') => Some(Action::CenterLine),
                _ => None,
            },
            _ => None,
        }
    }

    fn handle_insert_event(
        &mut self,
        code: &KeyCode,
        _modifiers: &KeyModifiers, // not used for now
    ) -> Result<Option<Action>> {
        let action = match code {
            KeyCode::Esc => Some(Action::EnterMode(Mode::Normal)),
            KeyCode::Backspace => Some(Action::RemoveChar),
            KeyCode::Enter => Some(Action::NewLine),
            KeyCode::Char(c) => Some(Action::AddChar(*c)),
            _ => None,
        };

        Ok(action)
    }

    fn handle_normal_event(
        &mut self,
        code: &KeyCode,
        modifiers: &KeyModifiers,
    ) -> Result<Option<Action>> {
        let action = match code {
            KeyCode::Char('z') => Some(Action::WaitingCmd('z')),
            KeyCode::Char('u') => Some(Action::Undo),
            KeyCode::Char(':') => Some(Action::EnterMode(Mode::Command)),

            // Insert Action
            KeyCode::Char('i') => Some(Action::EnterMode(Mode::Insert)),
            KeyCode::Char('a') => Some(Action::EnterMode(Mode::Insert)), //TODO Move cursor to
            //cursor right 1 time

            // Delete Action
            KeyCode::Char('x') => Some(Action::RemoveCharAt(self.v_cursor())),
            KeyCode::Char('d') => Some(Action::WaitingCmd('d')),

            // Create Action
            KeyCode::Char('o') => Some(Action::NewLineInsertionBelowCursor),
            KeyCode::Char('O') => Some(Action::NewLineInsertionAtCursor),

            //Movement Action
            KeyCode::PageUp => Some(Action::PageUp),
            KeyCode::PageDown => Some(Action::PageDown),
            KeyCode::Char('G') => Some(Action::EndOfFile),
            KeyCode::Char('g') => Some(Action::WaitingCmd('g')),
            KeyCode::Char('$') | KeyCode::End => Some(Action::EndOfLine),
            KeyCode::Char('0') | KeyCode::Home => Some(Action::StartOfLine),

            // Movement with Modifiers
            KeyCode::Char('f') if matches!(modifiers, &KeyModifiers::CONTROL) => {
                Some(Action::PageDown)
            }

            KeyCode::Char('b') if matches!(modifiers, &KeyModifiers::CONTROL) => {
                Some(Action::PageUp)
            }

            _ => None,
        };

        Ok(action)
    }

    fn handle_command_event(
        &mut self,
        code: &KeyCode,
        _modifiers: &KeyModifiers, // not used for now
    ) -> Result<Option<Action>> {
        let action = match code {
            KeyCode::Esc => Some(Action::EnterMode(Mode::Normal)),
            // KeyCode::Char('w') => Some(Action::SaveFile),
            KeyCode::Char(c) => Some(Action::AddCommandChar(*c)),
            KeyCode::Enter => {
                // handle the quit here to break the loop
                if self.command.as_str() == "q" {
                    return Ok(Some(Action::Quit));
                }
                Some(Action::ExecuteCommand)
            }
            KeyCode::Backspace => Some(Action::RemoveCommandChar),
            _ => None,
        };

        Ok(action)
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

    fn get_specific_line_len_by_mode(&mut self) -> u16 {
        // ive created this fn because the ll is different by the mode we are in
        // != Mode::Insert = ll - 1
        match self.viewport.get_line_len(&self.cursor) {
            0 => 0,
            ll if matches!(self.mode, Mode::Insert) => ll,
            ll => ll - TERMINAL_LINE_LEN_MINUS,
        }
    }
}

impl Draw for Editor {
    fn draw(&mut self) -> Result<()> {
        // some terminal line windows default show the cursor when drawing the tui so hide and show
        // it at the end of draw
        self.stdout.queue(cursor::Hide)?;
        self.viewport.draw(&mut self.stdout)?;
        self.draw_bottom()?;
        self.stdout
            .queue(cursor::MoveTo(self.cursor.0, self.cursor.1))?;

        self.stdout.queue(cursor::Show)?;
        self.stdout.flush()?;
        Ok(())
    }

    fn draw_bottom(&mut self) -> anyhow::Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, self.size.1 - TERMINAL_SIZE_MINUS))?;

        let cursor_viewport = self.viewport.viewport_cursor(&self.cursor);

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
        let cmd = &self.command;
        let r_width = self.size.0 as usize - cmd.len();
        self.stdout
            .queue(cursor::MoveTo(0, self.size.1 - 1))?
            .queue(PrintStyledContent(
                format!(":{cmd:<width$}", width = r_width - 1).on(colors::BG_0),
            ))?;
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
