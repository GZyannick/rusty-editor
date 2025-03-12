use crate::theme::colors;
use crate::viewport::{BufferPosition, Viewport, LINE_NUMBERS_WIDTH};
use crossterm::style::Color;

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
    fn buffer_current_position(&mut self) {
        self.buffer_position =
            BufferPosition::from(self.vwidth, self.vheight, self.min_vwidth, self.min_vheight)
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

        self.vwidth = self.buffer_position.width;
        self.vheight = self.buffer_position.height;
        self.min_vwidth = self.buffer_position.min_vwidth;
        self.min_vheight = self.buffer_position.min_vheight;
        self.buffer_position = BufferPosition::new();
        self.bg_color = Color::from(colors::DARK0);
        self.is_popup = false;
    }
}

#[cfg(test)]
mod test_popup {

    use crate::{
        buff::Buffer,
        languages::Languages,
        viewport::{BufferPosition, Viewport},
    };
    use crossterm::style::Color;

    fn create_test_viewport() -> Viewport {
        Viewport {
            buffer: Buffer::new(None),
            vwidth: 50,
            vheight: 20,
            min_vwidth: 5,
            min_vheight: 5,
            buffer_position: BufferPosition::new(),
            modifiable: true,
            left: 0,
            top: 0,
            languages: Languages::new(),
            // query: Query::new(&tree_sitter_rust::LANGUAGE.into(), HIGHLIGHTS_QUERY)
            // .expect("Query Error"),
            bg_color: Color::Black,
            is_popup: false,
            search_pos: vec![],
            search_index: 0,
            cached_highlight: None,
            last_highlighted_code: String::new(),
        }
    }

    #[test]
    fn test_as_popup() {
        let mut viewport = create_test_viewport();

        // Calling as_popup should update viewport properties
        viewport.as_popup();

        assert_eq!(viewport.vwidth, 35);
        assert_eq!(viewport.vheight, 14);
        assert_eq!(viewport.min_vwidth, 12);
        assert_eq!(viewport.min_vheight, 3);
        assert!(viewport.is_popup);
    }

    #[test]
    fn test_as_normal() {
        let mut viewport = create_test_viewport();

        // First, set it to popup mode
        viewport.as_popup();

        // Calling as_normal should reset to the original values
        viewport.as_normal();
        assert_eq!(viewport.vwidth, 50);
        assert_eq!(viewport.vheight, 20);
        assert_eq!(viewport.min_vwidth, 5);
        assert_eq!(viewport.min_vheight, 5);
        assert!(!viewport.is_popup);
    }
}
