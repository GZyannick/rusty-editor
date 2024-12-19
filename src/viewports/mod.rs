use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Ref, RefCell, RefMut},
};

use crate::viewport::Viewport;

#[derive(Debug)]
pub struct Viewports {
    pub values: Vec<Viewport>,
    pub index: usize,
}

impl Viewports {
    pub fn new() -> Viewports {
        Viewports {
            values: vec![],
            index: 0,
        }
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
}
