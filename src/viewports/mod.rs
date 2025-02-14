use crate::{buff::Buffer, editor::TERMINAL_SIZE_MINUS, viewport::Viewport};

#[derive(Debug)]
pub struct Viewports {
    pub values: Vec<Viewport>,
    pub index: usize,
    pub buffer_index: usize,
}

impl Viewports {
    pub fn new() -> Viewports {
        Viewports {
            values: vec![],
            index: 0,
            buffer_index: 0,
        }
    }

    // let us know if some viewport are save
    pub fn viewports_save_status(&mut self) -> anyhow::Result<bool> {
        for viewport in &mut self.values {
            if viewport.buffer.compare_file()? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn push(&mut self, viewport: Viewport) {
        self.values.push(viewport);
    }

    pub fn c_viewport(&self) -> &Viewport {
        self.values.get(self.index).unwrap()
    }

    pub fn c_mut_viewport(&mut self) -> &mut Viewport {
        self.values.get_mut(self.index).unwrap()
    }

    pub fn set_current_to_file_explorer_viewport(&mut self) {
        if let Some(pos) = self.values.iter().position(|v| v.is_file_explorer()) {
            self.buffer_index = self.index;
            self.index = pos;
        }
    }

    pub fn set_current_to_original_viewport(&mut self) {
        self.index = self.buffer_index;
        self.buffer_index = 0;
    }

    pub fn get_original_viewport(&mut self) -> Option<&mut Viewport> {
        self.get_by_index(self.buffer_index)
    }

    fn get_by_index(&mut self, index: usize) -> Option<&mut Viewport> {
        self.values.get_mut(index)
    }
}

impl Default for Viewports {
    fn default() -> Self {
        let values = vec![
            Viewport::new(Buffer::new(None), 80, 20, 0, true),
            Viewport::new(Buffer::new(Some("./".to_string())), 80, 20, 0, true),
        ];
        Self {
            values,
            index: 0,
            buffer_index: 0,
        }
    }
}
