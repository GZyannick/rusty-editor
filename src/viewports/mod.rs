use crate::{buff::Buffer, editor::TERMINAL_SIZE_MINUS, viewport::Viewport};
pub mod draw;
#[derive(Debug)]
pub struct Viewports {
    pub explorer: Viewport,
    pub values: Vec<Viewport>,
    pub index: usize,
    pub buffer_index: usize,
    pub is_explorer: bool,
}

impl Viewports {
    pub fn new(explorer: Viewport) -> Viewports {
        Viewports {
            explorer,
            values: vec![],
            index: 0,
            buffer_index: 0,
            is_explorer: false,
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
        match self.is_explorer {
            true => &self.explorer,
            false => self.values.get(self.index).unwrap(),
        }
    }

    pub fn c_mut_viewport(&mut self) -> &mut Viewport {
        match self.is_explorer {
            true => &mut self.explorer,
            false => self.values.get_mut(self.index).unwrap(),
        }
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
        let values = vec![Viewport::new(Buffer::new(None), 80, 20, 0, true)];
        Self {
            explorer: Viewport::new(Buffer::new(Some("./".to_string())), 80, 20, 0, true),
            values,
            index: 0,
            buffer_index: 0,
            is_explorer: false,
        }
    }
}
