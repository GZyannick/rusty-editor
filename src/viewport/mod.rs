mod ui;

use crossterm::{
    cursor,
    style::{PrintStyledContent, Stylize},
    QueueableCommand,
};

use crate::{buff::Buffer, editor, log_message, theme::colors};

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
                .get(self.top as usize + i as usize)
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

    // retrieve the len of the line
    pub fn get_line_len(&self, cursor: &(u16, u16)) -> u16 {
        let (_, y) = self.viewport_cursor(cursor);
        match self.buffer.get(y as usize) {
            Some(line) => line.len() as u16,
            None => 0,
        }
    }

    pub fn viewport_cursor(&self, cursor: &(u16, u16)) -> (u16, u16) {
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
        let (_, y) = self.viewport_cursor(cursor);
        (y as usize) < (self.buffer.lines.len() - 1_usize)
    }

    pub fn center_line(&mut self, cursor: &mut (u16, u16)) {
        let c_y = cursor.1;
        let half = self.vheight / 2;
        let v_cursor = self.viewport_cursor(cursor);
        match (c_y) < half {
            true => {
                // top half
                let move_len = half - c_y;
                if v_cursor.1 > half {
                    cursor.1 = half;
                    self.top -= move_len;
                }
            }
            false => {
                // bottom half
                let move_len = c_y - half;
                let buffer_len = self.get_buffer_len();
                if let Some(max_down) = buffer_len.checked_sub(v_cursor.1 as usize) {
                    if max_down > half as usize {
                        cursor.1 = half;
                        self.top += move_len;
                    }
                }
            }
        }
    }

    pub fn get_buffer_len(&self) -> usize {
        if self.buffer.lines.is_empty() {
            return 0;
        }
        self.buffer.lines.len()
    }
}
