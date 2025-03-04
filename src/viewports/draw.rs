use anyhow::Result;
use std::io::Write;

use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

use super::Viewports;
use crate::{editor, theme::colors, LINE_NUMBERS_WIDTH};

impl Viewports {
    // draw the name of each viewports at the top /
    pub fn draw<W: Write>(&self, stdout: &mut W, width: u16) -> Result<()> {
        let mut x = LINE_NUMBERS_WIDTH;
        for (i, v) in self.values.iter().enumerate() {
            let name = format!("  {}  ", v.buffer.path);
            let len = name.len();

            let name_color = match i == self.index {
                true => Color::from(colors::BRIGHT_YELLOW),
                false => Color::from(colors::GRAY_245),
            };

            // stop printing viewport if the size is > to the width of the terminal
            if x > width - LINE_NUMBERS_WIDTH - len as u16 {
                break;
            }

            stdout
                .queue(cursor::MoveTo(x, 0))?
                .queue(PrintStyledContent(
                    name.with(name_color).on(Color::from(colors::DARK0_SOFT)),
                ))?;

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
