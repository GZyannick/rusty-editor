mod core;
mod ui;

use crossterm::style::Color;
use tree_sitter::{Language, Query};
use tree_sitter_rust::HIGHLIGHTS_QUERY;

use crate::{buff::Buffer, theme::colors::DARK0};

const LINE_NUMBERS_WIDTH: u16 = 5;
// to implement scrolling and showing text of the size of our current terminal
#[derive(Debug)]
pub struct Viewport {
    pub buffer: Buffer,
    pub left: u16,
    pub top: u16,
    pub min_vwidth: u16,
    pub min_vheight: u16,
    // buffer position is when viewport change from its original position like popup, left -> right
    // and if we want to retrieve its old position we use the buffer_position
    //                    vw , vh , mvw, mvh
    pub buffer_position: (u16, u16, u16, u16),
    pub vwidth: u16,
    pub vheight: u16,
    pub query: Query,
    pub language: Language,
    pub bg_color: Color,
    pub is_popup: bool,
}

impl Viewport {
    pub fn new(buffer: Buffer, vwidth: u16, vheight: u16, min_vwidth: u16) -> Viewport {
        let language = tree_sitter_rust::LANGUAGE;
        // i am in obligation to put the Query::new in viewport or it will make lag the app
        // and make it unspossible to use tree_sitter without delay in the input
        let min_vwidth = min_vwidth + LINE_NUMBERS_WIDTH;
        Viewport {
            buffer,
            vwidth,
            vheight,
            min_vwidth,
            min_vheight: 0,
            left: 0,
            top: 0,
            buffer_position: (0, 0, 0, 0),
            language: language.into(),
            query: Query::new(&language.into(), HIGHLIGHTS_QUERY).expect("Query Error"),
            bg_color: Color::from(DARK0),
            is_popup: false,
        }
    }

    fn viewport(&self) -> String {
        if self.buffer.lines.is_empty() {
            return String::new();
        }

        let height = std::cmp::min((self.top + self.vheight) as usize, self.get_buffer_len());
        let vec = &self.buffer.lines;
        vec[self.top as usize..height].join("\n")
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

    pub fn is_under_buffer_len(&self, cursor: &(u16, u16)) -> bool {
        if self.buffer.lines.is_empty() {
            return false;
        }
        let (_, y) = self.viewport_cursor(cursor);
        (y as usize) < (self.buffer.lines.len() - 1_usize)
    }

    pub fn get_buffer_len(&self) -> usize {
        match self.buffer.lines.is_empty() {
            true => 0,
            false => self.buffer.lines.len(),
        }
    }
}
