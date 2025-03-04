use crate::{buff::Buffer, viewport::Viewport};
pub mod draw;
#[derive(Debug)]
pub struct Viewports {
    pub explorer: Viewport,
    pub values: Vec<Viewport>,
    pub index: usize,
    pub is_explorer: bool,
}

impl Viewports {
    pub fn new(explorer: Viewport) -> Viewports {
        Viewports {
            explorer,
            values: vec![],
            index: 0,
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

    pub fn push(&mut self, viewport: Viewport) -> usize {
        self.values.push(viewport);
        self.values.len() - 1
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

    pub fn prev_viewport(&mut self) {
        let prev_index = self.index.saturating_sub(1);
        self.index = match self.index == 0 {
            true => self.values.len() - 1,
            false => prev_index,
        }
    }
    pub fn next_viewport(&mut self) {
        let next_index = self.index + 1;
        self.index = match self.values.get(next_index).is_some() {
            true => next_index,
            false => 0,
        }
    }
}

impl Default for Viewports {
    fn default() -> Self {
        let values = vec![Viewport::new(Buffer::new(None), 80, 20, 0, true)];
        Self {
            explorer: Viewport::new(Buffer::new(Some("./".to_string())), 80, 20, 0, true),
            values,
            index: 0,
            is_explorer: false,
        }
    }
}
