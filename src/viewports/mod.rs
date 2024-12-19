use std::{
    borrow::{Borrow, BorrowMut},
    cell::{Ref, RefCell, RefMut},
};

use crate::viewport::Viewport;

#[derive(Debug)]
pub struct Viewports {
    pub values: RefCell<Vec<Viewport>>,
    pub current_viewport_index: usize,
}

impl Viewports {
    pub fn new() -> Viewports {
        Viewports {
            values: RefCell::new(vec![]),
            current_viewport_index: 0,
        }
    }

    pub fn push(&mut self, viewport: Viewport) {
        self.values.borrow_mut().push(viewport);
    }

    pub fn current_viewport(&self) -> Ref<Viewport> {
        let index = self.current_viewport_index;
        Ref::map(self.values.borrow(), |v| &v[index])
        // self.values.get(self.current_viewport_index).unwrap()
    }

    pub fn current_viewport_mut(&mut self) -> RefMut<Viewport> {
        let index = self.current_viewport_index;
        RefMut::map(self.values.borrow_mut(), |v| &mut v[index])
    }
}

#[cfg(test)]
mod tests {
    use crate::buff::Buffer;

    use super::*;

    #[test]
    fn test_get_viewport() {
        let viewport1 = Viewport::new(Buffer::new(None), 50, 50, 0);
        let viewport2 = Viewport::new(Buffer::new(Some(String::from("."))), 80, 90, 0);

        let mut viewports = Viewports::new();
        {
            viewports.push(viewport1);
            viewports.push(viewport2);

            let mut current_viewport = viewports.current_viewport_mut();
            assert_eq!(current_viewport.vwidth, 50);
            assert_eq!(current_viewport.vheight, 50);
            current_viewport.vwidth = 100;
        }

        {
            let new_current_viewport = viewports.current_viewport();
            assert_eq!(new_current_viewport.vwidth, 100);
        }
    }
}
