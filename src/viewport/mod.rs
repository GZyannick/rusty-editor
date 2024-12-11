mod core;
mod ui;

use streaming_iterator::StreamingIterator;

use crossterm::{
    cursor,
    style::{Color, PrintStyledContent, Stylize},
    QueueableCommand,
};
use tree_sitter::{Language, Parser, Query, QueryCursor};
use tree_sitter_rust::HIGHLIGHTS_QUERY;

use crate::{
    buff::Buffer,
    theme::{color_highligther::ColorHighligter, colors},
};

// to implement scrolling and showing text of the size of our current terminal
#[derive(Debug)]
pub struct Viewport {
    pub buffer: Buffer,
    pub left: u16,
    pub top: u16,
    pub vwidth: u16,
    pub vheight: u16,
    pub query: Query,
    pub language: Language,
}

impl Viewport {
    pub fn new(buffer: Buffer, vwidth: u16, vheight: u16) -> Viewport {
        let language = tree_sitter_rust::LANGUAGE;
        // i am in obligation to put the Query::new in viewport or it will make lag the app
        // and make it unspossible to use tree_sitter without delay in the input
        Viewport {
            buffer,
            vwidth,
            vheight,
            left: 0,
            top: 0,
            language: language.into(),
            query: Query::new(&language.into(), HIGHLIGHTS_QUERY).expect("Query Error"),
        }
    }

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

    fn viewport(&self) -> String {
        if self.buffer.lines.is_empty() {
            return String::new();
        }

        let height = std::cmp::min((self.top + self.vheight) as usize, self.get_buffer_len());
        let vec = &self.buffer.lines;
        vec[self.top as usize..height].join("\n")
    }

    pub fn draw(&mut self, stdout: &mut std::io::Stdout) -> anyhow::Result<()> {
        if self.buffer.lines.is_empty() {
            return Ok(());
        }

        let v_width = self.vwidth;
        stdout.queue(cursor::MoveTo(0, 0))?;
        let viewport_buffer = self.viewport();
        let colors = self.highlight(&viewport_buffer)?;

        let mut y: u16 = 0;
        let mut x: u16 = 0;
        let mut colorhighligter = None;

        for (pos, c) in viewport_buffer.chars().enumerate() {
            if c == '\n' {
                stdout
                    .queue(cursor::MoveTo(x, y))?
                    .queue(PrintStyledContent(
                        " ".repeat(v_width as usize).on(Color::from(colors::DARK0)),
                    ))?;
                x = 0;
                y += 1;
                continue;
            }
            if let Some(colorh) = colors.iter().find(|ch| pos == ch.start) {
                colorhighligter = Some(colorh);
            } else if colors.iter().find(|ch| pos == ch.end).is_some() {
                colorhighligter = None
            }

            let styled_char = match colorhighligter {
                Some(ch) => format!("{c}").on(Color::from(colors::DARK0)).with(ch.color),
                None => format!("{c}",).on(Color::from(colors::DARK0)),
            };

            stdout
                .queue(cursor::MoveTo(x, y))?
                .queue(PrintStyledContent(styled_char))?;
            x += 1;
        }

        Ok(())
    }

    fn draw_line_number(&self, stdout: &mut std::io::Stdout, i: u16) -> anyhow::Result<()> {
        let pos = self.top as usize + i as usize;

        let l_width = 4;
        stdout
            .queue(cursor::MoveTo(0, i))?
            .queue(PrintStyledContent(
                format!("{pos:>width$}", width = l_width).on(Color::from(colors::DARK0)),
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

    pub fn move_end(&mut self, cursor: &mut (u16, u16)) {
        let buffer_len = self.get_buffer_len() as u16;
        let vheight = self.vheight;
        if buffer_len > vheight {
            self.top = buffer_len - vheight;
            cursor.1 = vheight - 1;
        } else {
            cursor.1 = buffer_len - 1;
        }
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
