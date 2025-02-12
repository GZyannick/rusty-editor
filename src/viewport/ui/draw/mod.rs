use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};

use crate::viewport::LINE_NUMBERS_WIDTH;
use crate::{theme::colors, viewport::Viewport};

mod file;
mod file_explorer;
mod tree_highlight;
mod visual_block;
// implementing all draw fn in ui file
impl Viewport {
    pub fn draw(
        &self,
        stdout: &mut std::io::Stdout,
        start_v_mode: Option<(u16, u16)>,
        end_v_mode: Option<(u16, u16)>,
    ) -> anyhow::Result<()> {
        if self.buffer.lines.is_empty() {
            return Ok(());
        }

        //retrieve the last line position
        let y = match self.is_file_explorer() {
            true => file_explorer::draw_file_explorer(self, stdout)?,
            false => file::draw_file(self, stdout, start_v_mode, end_v_mode)?,
        };

        self.clear_end_of_viewport(y, stdout)?;

        Ok(())
    }

    // after draw line make sure that the rest of viewport is cleared
    // without ghostty text
    fn clear_end_of_viewport(&self, y: u16, stdout: &mut std::io::Stdout) -> anyhow::Result<()> {
        if y < self.vheight {
            for i in y..self.vheight {
                self.draw_line_number(stdout, i)?;
                stdout
                    .queue(cursor::MoveTo(self.min_vwidth - 1, i))?
                    .queue(PrintStyledContent(
                        " ".repeat(self.vwidth as usize).on(self.bg_color),
                    ))?;
            }
        }

        Ok(())
    }

    fn draw_line_number(&self, stdout: &mut std::io::Stdout, i: u16) -> anyhow::Result<()> {
        let pos = self.top as usize + i as usize;
        let l_width = LINE_NUMBERS_WIDTH as usize - 1;
        stdout
            .queue(cursor::MoveTo(self.min_vwidth - LINE_NUMBERS_WIDTH, i))?
            .queue(PrintStyledContent(
                format!("{pos:>width$}", width = l_width).on(self.bg_color),
            ))?;

        Ok(())
    }

    fn draw_search(&self, x: u16, y: u16) -> Option<Color> {
        if let Some(search_block) = self
            .search_pos
            .iter()
            .find(|&&(_, search_y, _)| search_y.saturating_sub(self.top) == y)
        {
            return Some(visual_block::draw_block(
                self,
                x,
                y,
                (search_block.0, search_block.1.saturating_sub(self.top)),
                (
                    search_block.0 + search_block.2.saturating_sub(1),
                    search_block.1.saturating_sub(self.top),
                ),
                Color::from(colors::BRIGHT_ORANGE),
            ));
        }

        None
    }
}
