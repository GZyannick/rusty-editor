use core::panic;

use crossterm::style::Color;

use crate::log_message;

use super::colors::{
    BRIGHT_AQUA, BRIGHT_BLUE, BRIGHT_GREEN, BRIGHT_PURPLE, BRIGHT_RED, BRIGHT_WHITE, BRIGHT_YELLOW,
    GRAY_245, NEUTRAL_AQUA, NEUTRAL_GREEN, NEUTRAL_RED, NEUTRAL_YELLOW,
};

#[derive(Debug, Clone, Copy)]
pub struct ColorHighligter {
    pub start: usize,
    pub end: usize,
    pub color: Color,
}

impl ColorHighligter {
    fn get_color_from_punctuation(punctuation: &str) -> Color {
        let color = match punctuation {
            "keyword" => NEUTRAL_RED,
            "punctuation.delimiter" => NEUTRAL_YELLOW,
            "punctuation.bracket" => NEUTRAL_YELLOW,
            "comment" => GRAY_245,
            "comment.documentation" => GRAY_245,
            "property" => BRIGHT_BLUE,
            "type" => BRIGHT_YELLOW,
            "type.builtin" => BRIGHT_YELLOW,
            "constructor" => BRIGHT_PURPLE,
            "attribute" => NEUTRAL_YELLOW,
            "variable.builtin" => BRIGHT_BLUE, // dont show up but exist
            "variable.parameter" => BRIGHT_BLUE,
            "constant.builtin" => BRIGHT_PURPLE,
            "function.method" => BRIGHT_GREEN,
            "function" => BRIGHT_RED,
            "operator" => NEUTRAL_YELLOW,
            "string" => NEUTRAL_GREEN,
            "function.macro" => NEUTRAL_AQUA,
            "escape" => NEUTRAL_YELLOW,

            "label" => BRIGHT_AQUA,
            // "identifier" => NEUTRAL_RED,
            // "punctuation.function" => NEUTRAL_GREEN,
            // "function" => NEUTRAL_GREEN,
            // "property" => BRIGHT_RED,
            // "attribute" => NEUTRAL_BLUE,
            // "type.builtin" => BRIGHT_YELLOW,
            // "variable.builtin" => BRIGHT_WHITE,
            // "punctuation.macro" => FADED_GREEN,
            // "punctuation.builtin" => BRIGHT_GREEN,
            // "keyword" => FADED_BLUE,
            // "constructor" => FADED_RED,
            // "type" => BRIGHT_YELLOW,
            _ => {
                log_message!("not used: {punctuation}");
                panic!();
                BRIGHT_WHITE
            }
        };

        Color::from(color)
    }

    pub fn new_from_capture(start: usize, end: usize, punctuation: &str) -> ColorHighligter {
        let color = Self::get_color_from_punctuation(punctuation);
        ColorHighligter { start, end, color }
    }
}
