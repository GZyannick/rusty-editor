use std::path::PathBuf;

use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, QueryCursor};

use crate::viewport::LINE_NUMBERS_WIDTH;
use crate::{
    theme::{color_highligther::ColorHighligter, colors},
    viewport::Viewport,
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
        for (i, line) in self.buffer.lines.iter().enumerate() {
            self.draw_line_number(stdout, y)?;

            let icon = match PathBuf::from(line).is_dir() {
                true => " \u{f115}",
                false => match line.split('.').last() {
                    Some("txt") => " \u{f15c}",
                    Some("md") => " \u{f48a}",
                    Some("rs") => " \u{e7a8}",
                    Some("py") => " \u{e73c}",
                    Some("png") | Some("jpg") => " \u{f1c5}",
                    _ => " \u{f016}",
                },
            };

            // we skip the ../ line
            //            // we skip the ../ line
            let line = match i > 0 {
                true if line.starts_with(self.buffer.path.as_str()) => {
                    let mut path = self.buffer.path.clone();
                    if !path.ends_with("/") {
                        path.push('/');
                    }
                    line.replacen(path.as_str(), "", 1).to_string()
                }
                true => line.to_string(),
                false => line.to_string(),
            };

            let path = format!(" {:<width$} ", line, width = self.vwidth as usize - 4);
            stdout
                .queue(cursor::MoveTo(self.min_vwidth - 1, y))?
                .queue(PrintStyledContent(
                    icon.with(Color::White).on(self.bg_color).bold(),
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

        // let chars_len = viewport_buffer.len().wrapping_sub(1);
        let chars_len = viewport_buffer.len().saturating_sub(1);
        let mut bg_color = self.bg_color;

        for (pos, c) in viewport_buffer.chars().enumerate() {
            // tell us that we are at the end of the line
            // so we draw the line number and empty char to end of terminal size to get the same bg
            // and dont have undesirable artifact like ghost char
            if c == '\n' {
                self.draw_line_number(stdout, y)?;
                stdout.queue(cursor::MoveTo(x + self.min_vwidth, y))?;
                if x < self.vwidth {
                    stdout.queue(PrintStyledContent(
                        " ".repeat(v_width as usize - x as usize).on(self.bg_color),
                    ))?;
                }
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

            // allow us to change the bg_color to draw the visual_block
            if let Some(start_block) = start_v_mode {
                if let Some(end_block) = end_v_mode {
                    bg_color = self.draw_block(
                        x,
                        y,
                        start_block,
                        end_block,
                        Color::from(colors::LIGTH_GREY),
                    );
                }
            }

            // if we are in search mode and we found occurences draw them
            if !self.search_pos.is_empty() {
                if let Some(color) = self.draw_search(x, y) {
                    bg_color = color
                }
            }

            // change char color if its highlight
            let styled_char = match colorhighligter {
                Some(ch) => format!("{c}").on(bg_color).with(ch.color),
                None => format!("{c}",).on(bg_color),
            };

            // move cursor to draw the char
            stdout
                .queue(cursor::MoveTo(x + self.min_vwidth, y))?
                .queue(PrintStyledContent(styled_char))?;

            x += 1;

            // if we are at the end of the string
            if pos == chars_len {
                self.draw_line_number(stdout, y)?;
                stdout.queue(cursor::MoveTo(x + self.min_vwidth, y))?;
                if x < self.vwidth {
                    stdout.queue(PrintStyledContent(
                        " ".repeat(v_width as usize - x as usize).on(self.bg_color),
                    ))?;
                }
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

    fn draw_search(&self, x: u16, y: u16) -> Option<Color> {
        if let Some(search_block) = self
            .search_pos
            .iter()
            .find(|&&(_, search_y, _)| search_y.saturating_sub(self.top) == y)
        {
            return Some(self.draw_block(
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

    fn draw_block(
        &self,
        x: u16,
        y: u16,
        start_block: (u16, u16),
        end_block: (u16, u16),
        color: Color,
    ) -> Color {
        match y >= start_block.1 && y <= end_block.1 {
            true => {
                if y == start_block.1 && y == end_block.1 {
                    match x >= start_block.0 && x <= end_block.0 {
                        true => color,
                        false => self.bg_color,
                    }
                } else if y == start_block.1 {
                    match x >= start_block.0 {
                        true => color,
                        false => self.bg_color,
                    }
                } else if y == end_block.1 {
                    match x <= end_block.0 {
                        true => color,
                        false => self.bg_color,
                    }
                } else {
                    return color;
                }
            }
            false => self.bg_color,
        }
    }
}
