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

    fn new(width: u16, height: u16) -> Self {
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
    pub fn popup(buffer: Buffer, width: u16, height: u16) -> Viewport {
        let language = tree_sitter_rust::LANGUAGE;
        let popup = Popup::new(width, height);

        Viewport {
            buffer,
            vwidth: popup.width,
            vheight: popup.height,
            min_vwidth: popup.left,
            min_vheight: popup.top,
            left: 0,
            top: 0,
            language: language.into(),
            query: Query::new(&language.into(), HIGHLIGHTS_QUERY).expect("Query Error"),
            bg_color: Color::from(colors::DARK1),
        }
    }

    pub fn as_popup(&mut self) {
        let popup = Popup::new(self.vwidth, self.vheight);

        self.vwidth = popup.width;
        self.vheight = popup.height;
        self.min_vwidth = popup.left;
        self.min_vheight = popup.top;
        self.bg_color = Color::from(colors::DARK1);
    }
}
