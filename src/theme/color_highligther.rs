use crossterm::style::Color;

use crate::THEME;

#[derive(Debug, Clone, Copy)]
pub struct ColorHighligter {
    pub start: usize,
    pub end: usize,
    pub color: Color,
}

impl ColorHighligter {
    fn get_color_from_punctuation(punctuation: &str) -> Color {
        let color = match punctuation {
            "keyword" => THEME.neutral_red,
            "punctuation.delimiter" => THEME.neutral_yellow,
            "punctuation.bracket" => THEME.neutral_yellow,
            "comment" => THEME.gray,
            "comment.documentation" => THEME.gray,
            "property" => THEME.bright_blue,
            "type" => THEME.bright_yellow,
            "type.builtin" => THEME.bright_yellow,
            "constructor" => THEME.bright_purple,
            "attribute" => THEME.neutral_yellow,
            "variable.builtin" => THEME.bright_blue,
            "variable.parameter" => THEME.bright_blue,
            "constant.builtin" => THEME.bright_purple,
            "function.method" => THEME.bright_green,
            "function" => THEME.bright_red,
            "operator" => THEME.neutral_yellow,
            "string" => THEME.neutral_green,
            "function.macro" => THEME.neutral_aqua,
            "escape" => THEME.neutral_yellow,
            "label" => THEME.bright_aqua,
            _ => THEME.default,
        };

        Color::from(color)
    }

    pub fn new_from_capture(start: usize, end: usize, punctuation: &str) -> ColorHighligter {
        let color = Self::get_color_from_punctuation(punctuation);
        ColorHighligter { start, end, color }
    }
}
