mod ui;

use crossterm::{
    cursor,
    style::{PrintStyledContent, Stylize},
    QueueableCommand,
};

use crate::{buff::Buffer, log_message, theme::colors};

// to implement scrolling and showing text of the size of our current terminal
#[derive(Debug)]
pub struct Viewport {
    pub buffer: Buffer,
    pub left: u16,
    pub top: u16,
    pub vwidth: u16,
    pub vheight: u16,
}

impl Viewport {
    pub fn new(buffer: Buffer, vwidth: u16, vheight: u16) -> Viewport {
        Viewport {
            buffer,
            vwidth,
            vheight,
            left: 0,
            top: 0,
        }
    }

    pub fn draw(&mut self, stdout: &mut std::io::Stdout) -> anyhow::Result<()> {
        if self.buffer.lines.is_empty() {
            return Ok(());
        }

        let v_width = self.vwidth;
        stdout.queue(cursor::MoveTo(0, 0))?;

        for i in 0..self.vheight {
            let line: String = self
                .buffer
                .get_line(self.top as usize + i as usize)
                .unwrap_or_default();

            // See if this is the best opt
            // to move it at 3 instead or 0

            // self.draw_line_number(stdout, i)?;
            stdout
                .queue(cursor::MoveTo(0, i))?
                .queue(PrintStyledContent(
                    format!("{line:<width$}", width = v_width as usize).on(colors::BG_0),
                ))?;
        }

        Ok(())
    }

    fn draw_line_number(&self, stdout: &mut std::io::Stdout, i: u16) -> anyhow::Result<()> {
        let pos = self.top as usize + i as usize;

        let l_width = 4;
        stdout
            .queue(cursor::MoveTo(0, i))?
            .queue(PrintStyledContent(
                format!("{pos:>width$}", width = l_width).on(colors::BG_0),
            ))?;

        Ok(())
    }

    pub fn get_line_len(&self, cursor: &(u16, u16)) -> u16 {
        let (_, y) = self.get_cursor_viewport_position(cursor);
        match self.buffer.get_line(y as usize) {
            Some(line) => line.len() as u16,
            None => 0,
        }
    }

    pub fn get_cursor_viewport_position(&self, cursor: &(u16, u16)) -> (u16, u16) {
        (cursor.0 + self.left, cursor.1 + self.top)
    }

    pub fn scroll_up(&mut self) {
        if self.top > 0 {
            self.top -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.top += 1;
    }

    pub fn page_up(&mut self) {
        if self.top > self.vheight {
            self.top -= self.vheight;
        } else {
            self.move_top();
        };
    }

    pub fn move_top(&mut self) {
        self.top = 0;
    }

    pub fn move_end(&mut self) {
        self.top = (self.buffer.lines.len() as u16) - self.vheight;
    }

    pub fn page_down(&mut self, cursor: &(u16, u16)) {
        if self.is_under_buffer_len(&(cursor.0, cursor.1 + self.vheight)) {
            self.top += self.vheight;
        } else {
            // allow us to move at the end of the file if the cursor is under the number of
            // buffer_lines
            let rest_of_file_len = (self.buffer.lines.len() as u16 - 1) - self.top;
            if rest_of_file_len > 0
                && self.is_under_buffer_len(&(cursor.0, cursor.1 + rest_of_file_len - 1))
            {
                self.top += rest_of_file_len;
            }
        }
    }

    pub fn is_under_buffer_len(&self, cursor: &(u16, u16)) -> bool {
        if self.buffer.lines.is_empty() {
            return false;
        }
        let (_, y) = self.get_cursor_viewport_position(cursor);
        (y as usize) < (self.buffer.lines.len() - 1_usize)
    }
}
