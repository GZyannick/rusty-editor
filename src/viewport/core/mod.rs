use crossterm::style::Color;
use tree_sitter::Query;
use tree_sitter_rust::HIGHLIGHTS_QUERY;

use crate::{buff::Buffer, theme::colors};

use super::{Viewport, LINE_NUMBERS_WIDTH};

const POPUP_PERCENTAGE: u16 = 30;
pub trait Popup {
    fn percentage_of(n: u16) -> u16 {
        (n * POPUP_PERCENTAGE) / 100
    }
    fn wrapping_sub_by_percentage(n: u16) -> u16 {
        n.wrapping_sub(Viewport::percentage_of(n))
    }

    fn popup(buffer: Buffer, width: u16, height: u16) -> Viewport {
        let language = tree_sitter_rust::LANGUAGE;

        let min_vwidth = (Viewport::percentage_of(width) / 2) + LINE_NUMBERS_WIDTH;
        let vwidth = Viewport::wrapping_sub_by_percentage(width);

        let min_vheight = Viewport::percentage_of(height) / 2;
        let vheight = Viewport::wrapping_sub_by_percentage(height);

        Viewport {
            buffer,
            vwidth,
            vheight,
            min_vwidth,
            min_vheight,
            left: 0,
            top: 0,
            language: language.into(),
            query: Query::new(&language.into(), HIGHLIGHTS_QUERY).expect("Query Error"),
            bg_color: Color::from(colors::DARK1),
        }
    }
}
