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

#[cfg(test)]
mod tests_visual_block {
    use super::*;

    use crossterm::style::Color;

    fn create_viewport() -> Viewport {
        Viewport {
            vwidth: 80,
            vheight: 20,
            min_vwidth: 5,
            bg_color: Color::Black, // Background color is set to black
            ..Viewport::default()
        }
    }

    #[test]
    fn test_draw_block_within_block_area() {
        let viewport = create_viewport();

        // Coordinates inside the block
        let start_block = (5, 5);
        let end_block = (10, 10);
        let x = 6;
        let y = 6;
        let color = Color::Blue; // Block color is blue

        let result = draw_block(&viewport, x, y, start_block, end_block, color);

        assert_eq!(
            result, color,
            "draw_block() should return the block color inside the block area"
        );
    }

    #[test]
    fn test_draw_block_outside_block_area() {
        let viewport = create_viewport();

        // Coordinates outside the block
        let start_block = (5, 5);
        let end_block = (10, 10);
        let x = 4;
        let y = 4;
        let color = Color::Blue;

        let result = draw_block(&viewport, x, y, start_block, end_block, color);

        assert_eq!(
            result, viewport.bg_color,
            "draw_block() should return the background color outside the block area"
        );
    }

    #[test]
    fn test_draw_block_at_start_boundary() {
        let viewport = create_viewport();

        // Coordinates on the start boundary
        let start_block = (5, 5);
        let end_block = (10, 10);
        let x = 5;
        let y = 5;
        let color = Color::Blue;

        let result = draw_block(&viewport, x, y, start_block, end_block, color);

        assert_eq!(
            result, color,
            "draw_block() should return the block color on the start boundary"
        );
    }

    #[test]
    fn test_draw_block_at_end_boundary() {
        let viewport = create_viewport();

        // Coordinates on the end boundary
        let start_block = (5, 5);
        let end_block = (10, 10);
        let x = 10;
        let y = 10;
        let color = Color::Blue;

        let result = draw_block(&viewport, x, y, start_block, end_block, color);

        assert_eq!(
            result, color,
            "draw_block() should return the block color on the end boundary"
        );
    }

    #[test]
    fn test_draw_block_at_upper_and_lower_boundaries() {
        let viewport = create_viewport();

        // Coordinates on the vertical boundaries (upper and lower)
        let start_block = (5, 5);
        let end_block = (10, 10);
        let x = 7; // In the middle of the block
        let y = 5; // Upper boundary

        let color = Color::Green;
        let result_upper = draw_block(&viewport, x, y, start_block, end_block, color);
        assert_eq!(
            result_upper, color,
            "draw_block() should return the block color at the upper boundary"
        );

        let y = 10; // Lower boundary
        let result_lower = draw_block(&viewport, x, y, start_block, end_block, color);
        assert_eq!(
            result_lower, color,
            "draw_block() should return the block color at the lower boundary"
        );
    }
}
