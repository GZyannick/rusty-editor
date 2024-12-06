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
            "function" => Color::Red,
            "property" => Color::Cyan,
            "attribute" => Color::DarkYellow,
            "constant.builtin" => Color::DarkRed,
            "type.builtin" => Color::DarkMagenta,
            "variable.builtin" => Color::DarkCyan,
            "variable.parameter" => Color::DarkGreen,
            "comment" => Color::Grey,
            "punctuation.bracket" => Color::DarkBlue,
            "punctuation.delimiter" => Color::Yellow,
            "function.method" => Color::Red,
            "function.macro" => Color::Yellow,
            "punctuation.macro" => Color::Yellow,
            "punctuation.builtin" => Color::Red,
            "keyword" => Color::Green,
            "constructor" => Color::Blue,
            "type" => Color::Yellow,
            "operator" => Color::Yellow,
            "label" => Color::Yellow,
            "string" => Color::Green,
            s => {
                log_message!("{s}");
                Color::White
            }
        }
    }

    pub fn new_from_capture(start: usize, end: usize, punctuation: &str) -> Self {
        let color = Self::get_color_from_punctuation(punctuation);
        Self { start, end, color }
    }
}
