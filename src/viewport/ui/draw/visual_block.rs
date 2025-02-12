use crate::viewport::Viewport;
use crossterm::style::Color;

pub fn draw_block(
    viewport: &Viewport,
    x: u16,
    y: u16,
    start_block: (u16, u16),
    end_block: (u16, u16),
    color: Color,
) -> Color {
    match y >= start_block.1 && y <= end_block.1 {
        true => {
            if y == start_block.1 && y == end_block.1 {
                match x >= start_block.0 && x <= end_block.0 {
                    true => color,
                    false => viewport.bg_color,
                }
            } else if y == start_block.1 {
                match x >= start_block.0 {
                    true => color,
                    false => viewport.bg_color,
                }
            } else if y == end_block.1 {
                match x <= end_block.0 {
                    true => color,
                    false => viewport.bg_color,
                }
            } else {
                return color;
            }
        }
        false => viewport.bg_color,
    }
}
