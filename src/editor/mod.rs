pub mod core;
pub mod ui;

use crate::editor::fmt::Debug;
use crate::viewport::Viewport;
use crate::{buff::Buffer, viewports::Viewports};
use crate::{log_message, THEME};
use anyhow::{Ok, Result};
use core::actions::action::Action;
use core::keybind_manager::KeybindManagerV2;
use core::mode::Mode;
use crossterm::{
    event::{self, read},
    style::Color,
    terminal, ExecutableCommand, QueueableCommand,
};
use std::fmt;
use std::io::{stdout, Cursor, Stdout, Write};
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

impl PartialEq for CursorBlock {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

pub struct Editor<W: Write> {
    pub toast: Toast,
    pub mode: Mode,
    pub keybinds: KeybindManagerV2,
    pub command: String,
    pub search: String,
    pub stdout: W,
    pub size: (u16, u16),
    pub cursor: (u16, u16),
    pub visual_cursor: Option<(u16, u16)>,

    pub modal: Option<Box<dyn ModalContent<W>>>,
    pub buffer_x_cursor: u16,
    pub waiting_command: Option<char>,
    pub viewports: Viewports,
    pub buffer_actions: Vec<Action>, // allow us to buffer some action to make multiple of them in one time
    pub undo_actions: Vec<Action>,   // create a undo buffer where we put all the action we want
    pub undo_insert_actions: Vec<Action>, // when we are in insert mode all the undo at the same
                                     // place
                                     // PS i could do better on comment
}

impl<W: Write> Editor<W> {
    pub fn new(buffer: Buffer, stdout: W) -> Result<Editor<W>> {
        let size = terminal::size()?;
        let (explorer, viewport) = match buffer.is_directory {
            true => (
                Viewport::new(buffer, size.0, size.1 - TERMINAL_SIZE_MINUS, 0, true),
                Viewport::new(
                    Buffer::new(None),
                    size.0,
                    size.1 - TERMINAL_SIZE_MINUS,
                    0,
                    true,
                ),
            ),
            false => (
                Viewport::new(
                    Buffer::new(Some(String::from("./"))),
                    size.0,
                    size.1 - TERMINAL_SIZE_MINUS,
                    0,
                    true,
                ),
                Viewport::new(buffer, size.0, size.1 - TERMINAL_SIZE_MINUS, 0, true),
            ),
        };

        let mut viewports = Viewports::new(explorer);
        viewports.push(viewport);

        let mut keybinds = KeybindManagerV2::new();
        keybinds.init_keybinds();

        Ok(Editor {
            toast: Toast::new(),
            mode: Mode::Normal,
            keybinds,
            search: String::new(),
            command: String::new(),
            stdout,
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
            .execute(crossterm::style::SetBackgroundColor(Color::from(THEME.bg0)))?;
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

        if self.cursor.1 as usize >= self.viewports.c_viewport().get_buffer_len() {
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
            match self.cursor.1 >= self.viewports.c_viewport().max_vheight().saturating_sub(1) {
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

        match self.viewports.c_viewport().get_line_len(&self.v_cursor()) {
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

    pub fn set_modal(&mut self, modal: Box<dyn ModalContent<W>>) {
        self.modal = Some(modal)
    }

    pub fn is_viewport_modifiable(&mut self) -> bool {
        let modifiable = self.viewports.c_viewport().modifiable;
        if !modifiable {
            self.toast
                .error("Viewport cannot be modifiable".to_string());
        }
        modifiable
    }
}

impl<W: Write> Drop for Editor<W> {
    fn drop(&mut self) {
        let _ = self
            .stdout
            .execute(terminal::Clear(terminal::ClearType::Purge));
        let _ = self.stdout.execute(terminal::LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

impl Default for Editor<Stdout> {
    fn default() -> Self {
        let mut keybinds = KeybindManagerV2::new();
        keybinds.init_keybinds();
        Self {
            toast: Toast::new(),
            mode: Mode::Normal,
            keybinds,
            search: String::new(),
            command: String::new(),
            stdout: stdout(),
            size: (80, 20),
            cursor: (0, 0),
            visual_cursor: None,
            modal: None,
            buffer_x_cursor: 0,
            waiting_command: None,
            viewports: Viewports::default(),
            buffer_actions: vec![],
            undo_actions: vec![],
            undo_insert_actions: vec![],
        }
    }
}

impl Default for Editor<Cursor<Vec<u8>>> {
    fn default() -> Self {
        let mut keybinds = KeybindManagerV2::new();
        keybinds.init_keybinds();

        Self {
            toast: Toast::new(),
            mode: Mode::Normal,
            keybinds,
            search: String::new(),
            command: String::new(),
            stdout: Cursor::new(Vec::new()),
            size: (80, 20),
            cursor: (0, 0),
            visual_cursor: None,
            modal: None,
            buffer_x_cursor: 0,
            waiting_command: None,
            viewports: Viewports::default(),
            buffer_actions: vec![],
            undo_actions: vec![],
            undo_insert_actions: vec![],
        }
    }
}

impl<W: Write> Debug for Editor<W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Editor")
            .field("size", &self.size)
            .field("mode", &self.mode)
            .field("cursor", &self.cursor)
            .field("command", &self.command)
            .field("search", &self.search)
            .field("viewports", &self.viewports)
            .field("stdout", &"stdout (not debuggable)") // Just print a placeholder message for stdout
            .finish()
    }
}
