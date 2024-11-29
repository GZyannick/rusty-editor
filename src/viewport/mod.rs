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
    pub width: u16,
    pub height: u16,
}

impl Viewport {
    pub fn new(buffer: Buffer, width: u16, height: u16) -> Viewport {
        Viewport {
            buffer,
            width,
            height,
            left: 0,
            top: 0,
        }
    }

    pub fn draw(&mut self, stdout: &mut std::io::Stdout) -> anyhow::Result<()> {
        if self.buffer.lines.is_empty() {
            return Ok(());
        }

        let v_width = self.width as usize;
        stdout.queue(cursor::MoveTo(0, 0))?;

        for (i, line) in self.get_buffer_viewport().iter().enumerate() {
            stdout
                .queue(cursor::MoveTo(0, i as u16))?
                .queue(PrintStyledContent(
                    format!("{line:<width$}", width = v_width).on(colors::BG_0),
                ))?;
        }
        Ok(())
    }

    pub fn get_line_len(&self, cursor: &(u16, u16)) -> u16 {
        let (_, y) = self.get_cursor_viewport_position(cursor);

        match self.buffer.get_line(y.into()) {
            Some(line) => line.len() as u16,
            None => 0,
        }
    }

    pub fn get_buffer_viewport(&mut self) -> &[String] {
        let start = self.top as usize;
        let end = (self.top + self.height) as usize;
        match end > self.buffer.lines.len() {
            true => &self.buffer.lines[start..],
            false => &self.buffer.lines[start..end],
        }
    }

    pub fn get_cursor_viewport_position(&self, cursor: &(u16, u16)) -> (u16, u16) {
        (cursor.0 + self.left, cursor.1 + self.top)
    }

    pub fn scroll_down(&mut self) {
        self.top += 1;
    }

    pub fn scroll_up(&mut self) {
        if self.top > 0 {
            self.top -= 1;
        }
    }

    pub fn get_cursor_max_x_position(&self, cursor: &(u16, u16)) -> u16 {
        let ll = self.get_line_len(cursor);
        //log_message!("ll = {}, cx = {}", ll, cursor.0);

        match cursor.0 > ll {
            true => ll,
            false => cursor.0,
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
