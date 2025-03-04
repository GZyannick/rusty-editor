use std::io::Write;

use super::action::Action;
use crate::{
    buff::Buffer,
    editor::{Editor, TERMINAL_SIZE_MINUS},
    viewport::Viewport,
};

impl Action {
    pub fn viewport<W: Write>(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        match self {
            Action::PrevViewport => editor.viewports.prev_viewport(),
            Action::NextViewport => editor.viewports.next_viewport(),
            Action::DeleteViewport => {
                editor.viewports.values.remove(editor.viewports.index);
                let action = match editor.viewports.values.is_empty() {
                    true => Action::PushEmptyViewport,
                    false => Action::PrevViewport,
                };
                editor.buffer_actions.push(action);
            }
            Action::DeleteOtherViewport => {
                let viewport_to_keep = editor
                    .viewports
                    .values
                    .drain(..)
                    .nth(editor.viewports.index)
                    .unwrap();
                editor.viewports.values = vec![viewport_to_keep];
                editor.viewports.index = 0;
            }
            Action::PushEmptyViewport => {
                editor.viewports.push(Viewport::new(
                    Buffer::new(None),
                    editor.size.0,
                    editor.size.1 - TERMINAL_SIZE_MINUS,
                    0,
                    true,
                ));
            }

            _ => {}
        }

        Ok(())
    }
}
