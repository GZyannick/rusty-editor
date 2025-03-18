use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

use crate::THEME;

pub trait ClearDraw {
    fn clear_at<W: std::io::Write>(
        &mut self,
        stdout: &mut W,
        start_x: u16,
        start_y: u16,
        width: u16,
        height: u16,
    ) -> anyhow::Result<()> {
        stdout.queue(cursor::MoveTo(start_x, start_y))?;
        // add + 1 in case a percentage take the integer under
        let max_h = height + start_y + 1;
        let max_w = width + start_x + 1;

        for i in start_y..max_h {
            stdout
                .queue(PrintStyledContent(
                    " ".repeat(max_w as usize).on(Color::from(THEME.bg0)),
                ))?
                .queue(cursor::MoveTo(start_x, i))?;
        }
        Ok(())
    }
}
