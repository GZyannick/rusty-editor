use std::io::Write;

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
    pub fn draw<W: std::io::Write>(
        &self,
        stdout: &mut W,
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
    fn clear_end_of_viewport<W: std::io::Write>(
        &self,
        y: u16,
        stdout: &mut W,
    ) -> anyhow::Result<()> {
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

    fn draw_line_number<W: Write>(&self, stdout: &mut W, i: u16) -> anyhow::Result<()> {
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

#[cfg(test)]
mod tests_viewport_draw {
    use super::*;
    use std::io::Cursor;
    fn create_mock_stdout() -> Cursor<Vec<u8>> {
        Cursor::new(Vec::new()) // Create a new Cursor to capture the output
    }

    #[test]
    fn test_draw_empty_buffer() {
        let viewport = Viewport {
            buffer: crate::buff::Buffer::new(None),
            ..Viewport::default()
        };

        let mut mock_stdout = create_mock_stdout();
        let result = viewport.draw(&mut mock_stdout, None, None);
        assert!(
            result.is_ok(),
            "draw() devrait réussir même si le buffer est vide"
        );
    }

    #[test]
    fn test_draw_file_explorer() {
        let viewport = Viewport {
            buffer: crate::buff::Buffer::new(Some("./".to_string())),
            ..Viewport::default()
        };

        let mut mock_stdout = create_mock_stdout();
        let result = viewport.draw(&mut mock_stdout, None, None);
        assert!(
            result.is_ok(),
            "draw() devrait réussir en mode file explorer"
        );
    }

    #[test]
    fn test_clear_end_of_viewport() {
        let viewport = Viewport {
            vheight: 10,
            vwidth: 20,
            min_vwidth: 5,
            ..Viewport::default()
        };

        let mut mock_stdout = create_mock_stdout();
        let result = viewport.clear_end_of_viewport(5, &mut mock_stdout);
        assert!(result.is_ok(), "clear_end_of_viewport() devrait réussir");

        // Vérifier que les lignes au-delà de y=5 ont bien été effacées
    }

    #[test]
    fn test_draw_line_number() {
        let viewport = Viewport {
            top: 0,
            min_vwidth: 5,
            ..Viewport::default()
        };

        let mut mock_stdout = create_mock_stdout();
        let result = viewport.draw_line_number(&mut mock_stdout, 3);
        assert!(result.is_ok(), "draw_line_number() devrait réussir");
    }

    #[test]
    fn test_draw_search_no_match() {
        let viewport = Viewport {
            search_pos: vec![], // Aucun résultat de recherche
            ..Viewport::default()
        };

        let color = viewport.draw_search(10, 5);
        assert!(
            color.is_none(),
            "draw_search() devrait retourner None si aucune correspondance"
        );
    }

    #[test]
    fn test_draw_search_with_match() {
        let viewport = Viewport {
            search_pos: vec![(5, 5, 3)], // Correspondance trouvée
            ..Viewport::default()
        };

        let color = viewport.draw_search(5, 5);
        assert!(
            color.is_some(),
            "draw_search() devrait retourner Some(Color) si une correspondance est trouvée"
        );
    }
}
