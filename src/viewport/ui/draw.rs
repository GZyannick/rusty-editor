use crossterm::{
    cursor,
    style::{PrintStyledContent, Stylize},
    QueueableCommand,
};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, QueryCursor};

use crate::{
    log_message,
    theme::color_highligther::ColorHighligter,
    viewport::{Viewport, LINE_NUMBERS_WIDTH},
};
// implementing all draw fn in ui file
impl Viewport {
    // highlight the rust code with tree_sitter and tree_sitter_rust
    pub fn highlight(&self, code: &String) -> anyhow::Result<Vec<ColorHighligter>> {
        let mut colors: Vec<ColorHighligter> = vec![];
        let mut parser = Parser::new();
        parser.set_language(&self.language)?;

        let tree = parser.parse(code, None).expect("tree_sitter couldnt parse");
        let mut query_cursor = QueryCursor::new();
        let mut query_matches =
            query_cursor.matches(&self.query, tree.root_node(), code.as_bytes());
        while let Some(m) = query_matches.next() {
            for cap in m.captures {
                let node = cap.node;
                let punctuation = self.query.capture_names()[cap.index as usize];

                colors.push(ColorHighligter::new_from_capture(
                    node.start_byte(),
                    node.end_byte(),
                    punctuation,
                ))
            }
        }
        Ok(colors)
    }

    pub fn draw(&self, stdout: &mut std::io::Stdout) -> anyhow::Result<()> {
        if self.buffer.lines.is_empty() {
            return Ok(());
        }

        let v_width = self.vwidth;
        let viewport_buffer = self.viewport();
        let colors = self.highlight(&viewport_buffer)?;

        let mut y: u16 = self.min_vheight;

        let mut x: u16 = 0;
        let mut colorhighligter = None;

        let chars_len = viewport_buffer.len() - 1;

        for (pos, c) in viewport_buffer.chars().enumerate() {
            if c == '\n' {
                self.draw_line_number(stdout, y)?;
                stdout
                    .queue(cursor::MoveTo(x + self.min_vwidth, y))?
                    .queue(PrintStyledContent(
                        " ".repeat(v_width as usize - x as usize).on(self.bg_color),
                    ))?;
                x = 0;
                y += 1;
                continue;
            }

            if let Some(colorh) = colors.iter().find(|ch| pos == ch.start) {
                colorhighligter = Some(colorh);
            } else if colors.iter().any(|ch| pos == ch.end) {
                colorhighligter = None
            }

            let styled_char = match colorhighligter {
                Some(ch) => format!("{c}").on(self.bg_color).with(ch.color),
                None => format!("{c}",).on(self.bg_color),
            };

            stdout
                .queue(cursor::MoveTo(x + self.min_vwidth, y))?
                .queue(PrintStyledContent(styled_char))?;

            x += 1;
            if pos == chars_len {
                self.draw_line_number(stdout, y)?;
                stdout
                    .queue(cursor::MoveTo(x + self.min_vwidth, y))?
                    .queue(PrintStyledContent(
                        " ".repeat(v_width as usize - x as usize).on(self.bg_color),
                    ))?;
                y += 1
            }
        }

        // after draw line make sure that the rest of viewport is cleared
        // without ghostty text
        if y < self.vheight {
            self.clear_end_of_viewport(stdout, y, v_width as usize)?;
        }

        // draw the end of popup if the size of lines is under the popup size
        if self.is_popup && y < self.vheight {
            self.draw_popup_end(y, stdout)?;
        }

        Ok(())
    }

    fn clear_end_of_viewport(
        &self,
        stdout: &mut std::io::Stdout,
        y: u16,
        width: usize,
    ) -> anyhow::Result<()> {
        for i in y..self.vheight {
            stdout
                .queue(cursor::MoveTo(0, i))?
                .queue(PrintStyledContent(" ".repeat(width).on(self.bg_color)))?;
        }
        Ok(())
    }

    fn draw_popup_end(&self, mut y: u16, stdout: &mut std::io::Stdout) -> anyhow::Result<()> {
        while y < self.vheight {
            self.draw_line_number(stdout, y)?;
            stdout
                .queue(cursor::MoveTo(self.min_vwidth, y))?
                .queue(PrintStyledContent(
                    " ".repeat(self.vwidth as usize).on(self.bg_color),
                ))?;
            y += 1;
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
}
