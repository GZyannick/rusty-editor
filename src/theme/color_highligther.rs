use crossterm::style::Color;

use crate::log_message;

use super::colors::{
    BRIGHT_BLUE, BRIGHT_GREEN, BRIGHT_ORANGE, BRIGHT_PURPLE, BRIGHT_RED, BRIGHT_WHITE,
    BRIGHT_YELLOW, FADED_BLUE, FADED_GREEN, FADED_ORANGE, FADED_PURPLE, FADED_RED, GRAY_245,
    NEUTRAL_BLUE, NEUTRAL_GREEN, NEUTRAL_ORANGE, NEUTRAL_RED, NEUTRAL_YELLOW,
};

#[derive(Debug)]
pub struct ColorHighligter {
    pub start: usize,
    pub end: usize,
    pub color: Color,
}

impl ColorHighligter {
    fn get_color(colors: (usize, usize, usize)) -> Color {
        Color::Rgb {
            r: colors.0 as u8,
            g: colors.1 as u8,
            b: colors.2 as u8,
        }
    }

    fn get_color_from_punctuation(punctuation: &str) -> Color {
        let color = match punctuation {
            "keyword" => NEUTRAL_RED,
            "punctuation.delimiter" => NEUTRAL_YELLOW,
            "punctuation.bracket" => NEUTRAL_YELLOW,
            "comment" => GRAY_245,
            "property" => BRIGHT_BLUE,
            "type" => BRIGHT_YELLOW,
            "type.builtin" => BRIGHT_YELLOW,
            "constructor" => BRIGHT_PURPLE,
            // "identifier" => NEUTRAL_RED,
            // "punctuation.function" => NEUTRAL_GREEN,
            // "function" => NEUTRAL_GREEN,
            // "property" => BRIGHT_RED,
            // "attribute" => NEUTRAL_BLUE,
            // "constant.builtin" => BRIGHT_RED,
            // "type.builtin" => BRIGHT_YELLOW,
            // "variable.builtin" => BRIGHT_WHITE,
            // "variable.parameter" => BRIGHT_YELLOW,
            // "comment" => GRAY_245,
            // "punctuation.bracket" => FADED_ORANGE,
            // "punctuation.delimiter" => BRIGHT_ORANGE,
            // "function.method" => BRIGHT_RED,
            // "function.macro" => BRIGHT_GREEN,
            // "punctuation.macro" => FADED_GREEN,
            // "punctuation.builtin" => BRIGHT_GREEN,
            // "keyword" => FADED_BLUE,
            // "constructor" => FADED_RED,
            // "type" => BRIGHT_YELLOW,
            // "operator" => NEUTRAL_ORANGE,
            // "label" => FADED_PURPLE,
            // "string" => FADED_GREEN,
            _ => BRIGHT_WHITE,
        };

        ColorHighligter::get_color(color)
    }

    pub fn new_from_capture(start: usize, end: usize, punctuation: &str) -> Self {
        let color = Self::get_color_from_punctuation(punctuation);
        Self { start, end, color }
    }
}
