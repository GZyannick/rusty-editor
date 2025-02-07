pub mod core;
pub mod ui;

use crate::theme::colors;
use crate::viewport::Viewport;
use crate::{buff::Buffer, viewports::Viewports};
use anyhow::{Ok, Result};
use core::actions::action::Action;
use core::mode::Mode;
use crossterm::{
    event::{self, read},
    style::Color,
    terminal, ExecutableCommand, QueueableCommand,
};
use std::io::Stdout;
use ui::modal::modal_trait::ModalContent;
use ui::toast::Toast;
// TERMINAL_LINE_LEN_MINUS if we want the cursor to go behind the last char or stop before,
// 1: stop on char, 0: stop after the char
pub const TERMINAL_LINE_LEN_MINUS: u16 = 1;
pub const TERMINAL_SIZE_MINUS: u16 = 2; // we remove the size of the bottom status, command bar
                                        // are at the end of the line or start move to next or prev line

#[derive(Debug, Clone, Copy)]
pub struct CursorBlock {
    pub start: (u16, u16),
    pub end: (u16, u16),
}

#[derive(Debug)]
pub struct Editor {
    pub toast: Toast,
    pub mode: Mode,
    pub command: String,
    pub search: String,
    pub stdout: Stdout,
    pub size: (u16, u16),
    pub cursor: (u16, u16),
    pub visual_cursor: Option<(u16, u16)>,

    pub modal: Option<Box<dyn ModalContent>>,
    pub buffer_x_cursor: u16,
    pub waiting_command: Option<char>,
    pub viewports: Viewports,
    pub buffer_actions: Vec<Action>, // allow us to buffer some action to make multiple of them in one time
    pub undo_actions: Vec<Action>,   // create a undo buffer where we put all the action we want
    pub undo_insert_actions: Vec<Action>, // when we are in insert mode all the undo at the same
                                     // place
                                     // PS i could do better on comment
}

impl Editor {
    pub fn new(buffer: Buffer) -> Result<Editor> {
        let size = terminal::size()?;

        let mut viewports = Viewports::new();
        let mut explorer_viewport = Viewport::new(
            Buffer::new(Some(String::from("./"))),
            size.0,
            size.1 - TERMINAL_SIZE_MINUS,
            0,
        );

        if buffer.is_directory {
            explorer_viewport = Viewport::new(buffer, size.0, size.1 - TERMINAL_SIZE_MINUS, 0);

            // this is an empty file viewport
            viewports.push(Viewport::new(
                Buffer::new(None),
                size.0,
                size.1 - TERMINAL_SIZE_MINUS,
                0,
            ));

            // Viewport::new(Buffer::new(None), size.0, size.1 - TERMINAL_SIZE_MINUS, 0)
        } else {
            viewports.push(Viewport::new(
                buffer,
                size.0,
                size.1 - TERMINAL_SIZE_MINUS,
                0,
            ));
        }
        viewports.push(explorer_viewport);

        Ok(Editor {
            toast: Toast::new(),
            mode: Mode::Normal,
            search: String::new(),
            command: String::new(),
            stdout: std::io::stdout(),
            size,
            cursor: (0, 0),
            visual_cursor: None,
            modal: None,
            buffer_x_cursor: 0,
            waiting_command: None,
            viewports,
            buffer_actions: vec![],
            undo_actions: vec![],
            undo_insert_actions: vec![],
        })
    }

    pub fn is_visual_mode(&self) -> bool {
        matches!(self.mode, Mode::Visual)
    }
    // viewport cursor
    pub fn v_cursor(&self) -> (u16, u16) {
        self.viewports.c_viewport().viewport_cursor(&self.cursor)
    }

    pub fn enter_raw_mode(&mut self) -> anyhow::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        self.stdout
            .execute(crossterm::style::SetBackgroundColor(Color::from(
                colors::DARK0,
            )))?;
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
            if self.buffer_x_cursor == self.viewports.c_viewport().min_vwidth_without_line_number()
            {
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

        if self.v_cursor().1 as usize >= self.viewports.c_viewport().get_buffer_len() {
            self.cursor.1 = self.cursor.1.saturating_sub(1);
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
                if matches!(action, Action::ForceQuit)
                    || matches!(action, Action::Quit) && self.viewports.viewports_save_status()?
                {
                    break;
                }
                action.execute(self)?;
            }
        }
        Ok(())
    }

    fn resize(&mut self, w: u16, h: u16) -> Result<()> {
        // resize each viewport
        for viewport in &mut self.viewports.values {
            viewport.vwidth = w;
            viewport.vheight = h.saturating_sub(TERMINAL_SIZE_MINUS);
        }

        self.size = (w, h);
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?;

        Ok(())
    }

    fn move_prev_line(&mut self) {
        match self.cursor.1 > 0 {
            true => self.cursor.1 = self.cursor.1.saturating_sub(1),
            false => self.viewports.c_mut_viewport().scroll_up(),
        }
    }

    fn move_next_line(&mut self) {
        if self
            .viewports
            .c_viewport()
            .is_under_buffer_len(&self.cursor)
        {
            match self.cursor.1 >= (self.viewports.c_viewport().vheight - 1) {
                true => self.viewports.c_mut_viewport().scroll_down(),
                false => self.cursor.1 += 1,
            }
        }
    }

    // allow us to know with of cursor or visual_cursor is the first to come
    fn get_visual_block_pos(&self) -> Option<CursorBlock> {
        if let Some(visual_cursor) = self.visual_cursor {
            let (start, end) = match self.cursor.1.cmp(&visual_cursor.1) {
                std::cmp::Ordering::Less => (self.cursor, visual_cursor),
                std::cmp::Ordering::Equal => match self.cursor.0.cmp(&visual_cursor.0) {
                    std::cmp::Ordering::Less => (self.cursor, visual_cursor),
                    _ => (visual_cursor, self.cursor),
                },
                std::cmp::Ordering::Greater => (visual_cursor, self.cursor),
            };
            return Some(CursorBlock { start, end });
        };
        None
    }

    // we have to take the min size of the viewport in condition
    fn get_specific_line_len_by_mode(&mut self) -> u16 {
        // ive created this fn because the ll is different by the mode we are in
        // != Mode::Insert = ll - 1
        match self.viewports.c_viewport().get_line_len(&self.cursor) {
            0 => 0,
            ll if matches!(self.mode, Mode::Insert) => ll,
            ll => ll - TERMINAL_LINE_LEN_MINUS,
        }
    }

    fn reset_cursor(&mut self) {
        self.cursor = (0, 0);

        let c_mut_viewport = self.viewports.c_mut_viewport();
        c_mut_viewport.top = 0;
        c_mut_viewport.left = 0;
    }

    pub fn set_modal(&mut self, modal: Box<dyn ModalContent>) {
        self.modal = Some(modal)
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
