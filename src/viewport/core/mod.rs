use crossterm::style::Color;

use crate::log_message;

#[derive(Debug)]
pub struct ColorHighligter {
    pub start: usize,
    pub end: usize,
    pub color: Color,
}

impl ColorHighligter {
    fn get_color_from_punctuation(punctuation: &str) -> Color {
        match punctuation {
            "punctuation.function" => Color::Red,
            "punctuation.bracket" => Color::DarkBlue,
            "punctuation.delimiter" => Color::Yellow,
            "punctuation.macro" => Color::Yellow,
            "punctuation.builtin" => Color::Red,
            "keyword" => Color::Green,
            "constructor" => Color::Blue,
            "type" => Color::Yellow,
            "operator" => Color::Yellow,
            "label" => Color::Yellow,
            _ => Color::White,
        }
    }

    pub fn new_from_capture(start: usize, end: usize, punctuation: &str) -> Self {
        let color = Self::get_color_from_punctuation(punctuation);
        Self { start, end, color }
    }
}
