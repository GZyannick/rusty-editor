use crate::{editor::TERMINAL_LINE_LEN_MINUS, viewport::Viewport};

impl Viewport {
    pub fn scroll_up(&mut self) {
        self.top = self.top.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.top += 1;
    }
    pub fn scroll_left(&mut self) {
        self.left = self.left.saturating_sub(1);
    }
    pub fn scroll_right(&mut self) {
        self.left += 1;
    }

    pub fn page_up(&mut self) {
        if self.top > self.vheight {
            self.top -= self.vheight;
        } else {
            self.top = 0
        };
    }

    pub fn move_start_of_line(&mut self) {
        self.left = 0;
    }

    pub fn move_end_of_line(&mut self, cursor: &mut (u16, u16)) {
        let line_len = self.get_line_len(cursor);
        let max_vwidth = self.max_vwidth().saturating_sub(1);
        match line_len > max_vwidth {
            true => {
                self.calculate_left_and_cursor_position(cursor);
            }
            false => {
                cursor.0 = self
                    .get_line_len(cursor)
                    .wrapping_sub(TERMINAL_LINE_LEN_MINUS)
            }
        }
    }

    pub fn calculate_left_and_cursor_position(&mut self, cursor: &mut (u16, u16)) {
        let max_vwidth = self.max_vwidth().saturating_sub(1);
        let without_line_len_minus = self
            .get_line_len(cursor)
            .saturating_sub(TERMINAL_LINE_LEN_MINUS);

        cursor.0 = max_vwidth;
        self.left = without_line_len_minus.saturating_sub(max_vwidth);
    }

    pub fn check_left_bound(&mut self, cursor: &mut (u16, u16)) {
        if self.left > 0 && self.max_vwidth() > self.get_line_len(cursor) {
            self.calculate_left_and_cursor_position(cursor);
        }
    }
    pub fn move_top(&mut self, cursor: &mut (u16, u16)) {
        self.top = 0;
        cursor.1 = 0;
        self.check_left_bound(cursor);
    }

    pub fn move_end(&mut self, cursor: &mut (u16, u16)) {
        let buffer_len = self.get_buffer_len() as u16;
        let vheight = self.max_vheight();
        if buffer_len > vheight {
            self.top = buffer_len - vheight;
            cursor.1 = vheight - 1;
        } else {
            cursor.1 = buffer_len - 1;
        }
        self.check_left_bound(cursor);
    }

    pub fn max_vheight(&self) -> u16 {
        self.vheight.saturating_sub(self.min_vheight)
    }

    pub fn max_vwidth(&self) -> u16 {
        self.vwidth.saturating_sub(self.min_vwidth)
    }

    pub fn page_down(&mut self, cursor: &(u16, u16)) {
        if self.is_under_buffer_len(&(cursor.0, cursor.1 + self.vheight)) {
            self.top += self.vheight;
        } else {
            // allow us to move at the end of the file if the cursor is under the number of
            // buffer_lines
            let rest_of_file_len = (self.buffer.lines.len() as u16 - 1) - self.top;
            if rest_of_file_len > 0
                && self.is_under_buffer_len(&(cursor.0, cursor.1 + rest_of_file_len - 1))
            {
                self.top += rest_of_file_len;
            }
        }
    }

    pub fn move_to(&mut self, cursor: &(u16, u16)) -> (u16, u16) {
        // calculate the editor cursor position from an v_cursor
        let quotient = cursor.1 / self.max_vheight();
        let remain = cursor.1 % self.max_vheight();
        self.top = self.max_vheight() * quotient;
        (cursor.0, remain.saturating_sub(self.min_vheight))
    }

    pub fn center_line(&mut self, cursor: &mut (u16, u16)) {
        let c_y = cursor.1;
        let half = self.max_vheight() / 2;
        let v_cursor = self.viewport_cursor(cursor);
        match (c_y) < half {
            true => {
                // top half
                let move_len = half - c_y;
                if v_cursor.1 > half {
                    cursor.1 = half;
                    self.top -= move_len;
                }
            }
            false => {
                // bottom half
                let move_len = c_y - half;
                let buffer_len = self.get_buffer_len();
                if let Some(max_down) = buffer_len.checked_sub(v_cursor.1 as usize) {
                    if max_down > half as usize {
                        cursor.1 = half;
                        self.top += move_len;
                    }
                }
            }
        }

        self.check_left_bound(cursor);
    }
}
