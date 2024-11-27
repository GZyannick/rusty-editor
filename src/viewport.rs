use crate::buffer::Buffer;

// to implement scrolling and showing text of the size of our current terminal
#[derive(Debug)]
pub struct Viewport {
    pub buffer: Buffer,
    pub x_pos: u16, 
    pub y_pos: u16,
    pub width: u16,
    pub height: u16,
}

impl Viewport {
    pub fn new(buffer: Buffer, width: u16, height: u16) -> Viewport {
        Viewport {
            buffer,
            width,
            height,
            x_pos: 0,
            y_pos: 0,
        }
    }

   pub fn get_buffer_viewport(&mut self) -> &[String] {
        let start = self.y_pos as usize;
        let end = (self.y_pos + self.height) as usize;
        &self.buffer.lines[start..end]
   } 

   pub fn get_cursor_viewport_position(&self, cursor: &(u16, u16)) -> (u16, u16) {
       (cursor.0 + self.x_pos, cursor.1 + self.y_pos)
   }

   pub fn scroll_down(&mut self) {
        self.y_pos += 1;
   }
   
   pub fn scroll_up(&mut self) {
       if self.y_pos > 0 {
           self.y_pos -= 1;
       }
   }

   pub fn is_under_buffer_len(&self, cursor: &(u16, u16)) -> bool {
       if self.buffer.lines.is_empty() {
           return false;
       }
       let cursor_viewport_position = self.get_cursor_viewport_position(cursor);
       (cursor_viewport_position.1  as usize) < (self.buffer.lines.len() - 1_usize)
   }

}
