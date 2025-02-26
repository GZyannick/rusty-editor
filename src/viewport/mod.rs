pub mod core;
mod ui;

use std::collections::HashMap;

use crossterm::style::Color;
use tree_sitter::{Language, Query};
use tree_sitter_rust::HIGHLIGHTS_QUERY;

use crate::{
    buff::Buffer,
    languages::{self, Languages},
    theme::{color_highligther::ColorHighligter, colors::DARK0},
};

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

    pub modifiable: bool,
    pub vwidth: u16,
    pub vheight: u16,
    // pub query: Query,
    pub languages: Languages,
    pub bg_color: Color,
    pub is_popup: bool,
    // when we do some search it will store all position of match content
    pub search_pos: Vec<(u16, u16, u16)>, // x, y, len
    pub search_index: usize,              // to iter through search_pos;

    pub cached_highlight: Option<Vec<ColorHighligter>>,
    pub last_highlighted_code: String,
}

impl Viewport {
    pub fn new(
        mut buffer: Buffer,
        vwidth: u16,
        vheight: u16,
        min_vwidth: u16,
        modifiable: bool,
    ) -> Viewport {
        // i am in obligation to put the Query::new in viewport or it will make lag the app
        // and make it unspossible to use tree_sitter without delay in the input
        let min_vwidth = min_vwidth + LINE_NUMBERS_WIDTH;

        let languages = Languages::new();

        buffer.set_query_language(&languages);
        Viewport {
            buffer,
            modifiable,
            vwidth,
            vheight,
            min_vwidth,
            min_vheight: 0,
            left: 0,
            top: 0,
            buffer_position: (0, 0, 0, 0),
            languages,
            bg_color: Color::from(DARK0),
            is_popup: false,
            search_pos: vec![],
            search_index: 0,
            cached_highlight: None,
            last_highlighted_code: String::new(),
        }
    }

    // let us know if the viewport is the file_explorer.
    pub fn is_file_explorer(&self) -> bool {
        self.buffer.is_directory
    }

    // return a string with the size of the viewport
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

    // return the viewport cursor
    pub fn viewport_cursor(&self, cursor: &(u16, u16)) -> (u16, u16) {
        (cursor.0 + self.left, cursor.1 + self.top)
    }

    // let us know is the cursor is under the buffer max len
    pub fn is_under_buffer_len(&self, cursor: &(u16, u16)) -> bool {
        if self.buffer.lines.is_empty() {
            return false;
        }
        let (_, y) = self.viewport_cursor(cursor);
        (y as usize) < (self.buffer.lines.len() - 1_usize)
    }

    // return the buffer len
    pub fn get_buffer_len(&self) -> usize {
        match self.buffer.lines.is_empty() {
            true => 0,
            false => self.buffer.lines.len(),
        }
    }

    // let us find all occurence of the search
    pub fn find_occurence(&mut self, find: &str) {
        let mut occurences: Vec<(u16, u16, u16)> = vec![];
        if find.is_empty() {
            self.search_pos = occurences;
            return;
        }

        for (y, line) in self.buffer.lines.iter().enumerate() {
            for (x, _) in line.match_indices(find) {
                occurences.push((x as u16, y as u16, find.len() as u16));
            }
        }
        self.search_pos = occurences;
    }

    pub fn min_vwidth_without_line_number(&self) -> u16 {
        self.min_vwidth - LINE_NUMBERS_WIDTH
    }

    pub fn clear_search(&mut self) {
        self.search_index = 0;
        self.search_pos = vec![];
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Viewport {
            buffer: Buffer::new(None),
            modifiable: true,
            vwidth: 80,
            vheight: 20,
            min_vwidth: LINE_NUMBERS_WIDTH,
            min_vheight: 0,
            left: 0,
            top: 0,
            buffer_position: (0, 0, 0, 0),
            languages: Languages::new(),
            // query: Query::new(&language.into(), HIGHLIGHTS_QUERY).expect("Query Error"),
            bg_color: Color::from(DARK0),
            is_popup: false,
            search_pos: vec![],
            search_index: 0,
            cached_highlight: None,
            last_highlighted_code: String::new(),
        }
    }
}
