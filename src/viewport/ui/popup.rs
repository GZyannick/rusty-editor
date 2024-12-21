use crate::viewport::{Viewport, LINE_NUMBERS_WIDTH};
use crossterm::style::Color;
use tree_sitter::Query;
use tree_sitter_rust::HIGHLIGHTS_QUERY;

use crate::{buff::Buffer, theme::colors};

const POPUP_PERCENTAGE: u16 = 30;

pub struct Popup {
    pub width: u16,
    pub height: u16,
    pub top: u16,
    pub left: u16,
}

impl Popup {
    fn percentage_of(n: u16) -> u16 {
        (n * POPUP_PERCENTAGE) / 100
    }

    fn wrapping_sub_by_percentage(n: u16) -> u16 {
        n.wrapping_sub(Popup::percentage_of(n))
    }

    pub fn new(width: u16, height: u16) -> Self {
        let left = (Popup::percentage_of(width) / 2) + LINE_NUMBERS_WIDTH;
        let top = Popup::percentage_of(height) / 2;
        let width = Popup::wrapping_sub_by_percentage(width);
        let height = Popup::wrapping_sub_by_percentage(height);

        Popup {
            width,
            height,
            top,
            left,
        }
    }
}

impl Viewport {
    // Can be used later
    pub fn _popup(buffer: Buffer, width: u16, height: u16) -> Viewport {
        let language = tree_sitter_rust::LANGUAGE;
        let popup = Popup::new(width, height);

        Viewport {
            buffer,
            vwidth: popup.width,
            vheight: popup.height,
            min_vwidth: popup.left,
            min_vheight: popup.top,
            buffer_position: (0, 0, 0, 0),
            left: 0,
            top: 0,
            language: language.into(),
            query: Query::new(&language.into(), HIGHLIGHTS_QUERY).expect("Query Error"),
            bg_color: Color::from(colors::DARK1),
            is_popup: true,
        }
    }

    fn buffer_current_position(&mut self) {
        self.buffer_position = (self.vwidth, self.vheight, self.min_vwidth, self.min_vheight);
    }

    pub fn as_popup(&mut self) {
        if self.is_popup {
            return;
        }

        let popup = Popup::new(self.vwidth, self.vheight);
        self.buffer_current_position();

        self.vwidth = popup.width;
        self.vheight = popup.height;
        self.min_vwidth = popup.left;
        self.min_vheight = popup.top;
        self.bg_color = Color::from(colors::DARK1);
        self.is_popup = true;
    }

    pub fn as_normal(&mut self) {
        if !self.is_popup {
            return;
        }

        self.vwidth = self.buffer_position.0;
        self.vheight = self.buffer_position.1;
        self.min_vwidth = self.buffer_position.2;
        self.min_vheight = self.buffer_position.3;
        self.buffer_position = (0, 0, 0, 0);
        self.bg_color = Color::from(colors::DARK0);
        self.is_popup = false;
    }
}
