use crate::{log_message, viewport::Viewport};

impl Viewport {
    pub fn scroll_up(&mut self) {
        if self.top > 0 {
            self.top -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.top += 1;
    }

    pub fn page_up(&mut self) {
        if self.top > self.vheight {
            self.top -= self.vheight;
        } else {
            self.move_top();
        };
    }

    pub fn move_top(&mut self) {
        self.top = 0;
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
    }

    pub fn max_vheight(&self) -> u16 {
        self.vheight.saturating_sub(self.min_vheight)
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
    }
}
