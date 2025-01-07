use std::path::PathBuf;

use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, QueryCursor};

use crate::{
    theme::{color_highligther::ColorHighligter, colors},
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

    pub fn draw_file_explorer(&self, stdout: &mut std::io::Stdout) -> anyhow::Result<u16> {
        let mut y = self.min_vheight;
        for line in self.buffer.lines.iter() {
            self.draw_line_number(stdout, y)?;

            let icon: String = match PathBuf::from(line).is_dir() {
                true => "📁".to_string(),
                false => "📰".to_string(),
            };

            let path = format!(" {:<width$} ", line, width = self.vwidth as usize - 4);
            stdout
                .queue(cursor::MoveTo(self.min_vwidth - 1, y))?
                .queue(PrintStyledContent(
                    icon.with(Color::White).on(self.bg_color),
                ))?
                .queue(cursor::MoveTo(self.min_vwidth + 1, y))?
                .queue(PrintStyledContent(
                    path.with(Color::White).on(self.bg_color),
                ))?;
            y += 1;
        }
        Ok(y)
    }

    pub fn draw_file(
        &self,
        stdout: &mut std::io::Stdout,
        start_v_mode: Option<(u16, u16)>,
        end_v_mode: Option<(u16, u16)>,
    ) -> anyhow::Result<u16> {
        let v_width = self.vwidth;
        let viewport_buffer = self.viewport();
        let colors = self.highlight(&viewport_buffer)?;

        let mut y: u16 = self.min_vheight;

        let mut x: u16 = 0;
        let mut colorhighligter = None;

        if let Some(visual_block_start) = start_v_mode {}
        if let Some(visual_block_end) = end_v_mode {}

        let chars_len = viewport_buffer.len().wrapping_sub(1);
        let mut bg_color = self.bg_color;

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

            // let us know if the current char is part of an highlight
            if let Some(colorh) = colors.iter().find(|ch| pos == ch.start) {
                colorhighligter = Some(colorh);
            } else if colors.iter().any(|ch| pos == ch.end) {
                colorhighligter = None
            }

            if let Some(visual_block_start) = start_v_mode {
                if let Some(visual_block_end) = end_v_mode {
                    match y >= visual_block_start.1 && y <= visual_block_end.1 {
                        true => bg_color = Color::from(colors::LIGTH_GREY),
                        false => bg_color = self.bg_color,
                    }
                }
            }
            let styled_char = match colorhighligter {
                Some(ch) => format!("{c}").on(bg_color).with(ch.color),
                None => format!("{c}",).on(bg_color),
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
        Ok(y)
    }

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
            true => self.draw_file_explorer(stdout)?,
            false => self.draw_file(stdout, start_v_mode, end_v_mode)?,
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
}
