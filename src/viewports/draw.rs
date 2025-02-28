use anyhow::Result;
use std::io::Write;

use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

use super::Viewports;
use crate::{theme::colors, LINE_NUMBERS_WIDTH};

impl Viewports {
    // draw the name of each viewports at the top /
    pub fn draw<W: Write>(&self, stdout: &mut W, width: u16) -> Result<()> {
        let mut x = LINE_NUMBERS_WIDTH;

        // let pad_width = editor.size.0 - mode.len() as u16 - pos.len() as u16 - TERMINAL_SIZE_MINUS;
        for v in self.values.iter().filter(|v| !v.is_file_explorer()) {
            let name = format!("  {}  ", v.buffer.path);
            let len = name.len();

            // stop printing viewport if the size is > to the width of the terminal
            if x > width - LINE_NUMBERS_WIDTH - len as u16 {
                break;
            }

            stdout
                .queue(cursor::MoveTo(x, 0))?
                .queue(PrintStyledContent(name.on(Color::from(colors::DARK0_SOFT))))?;

            x += len as u16;
        }

        stdout
            .queue(cursor::MoveTo(x, 0))?
            .queue(PrintStyledContent(
                " ".repeat(width as usize - x as usize - LINE_NUMBERS_WIDTH as usize)
                    .on(Color::from(colors::DARK1)),
            ))?;

        Ok(())
    }
}
