use std::io::Write;

use super::action::Action;
use crate::editor::Editor;

impl Action {
    pub fn viewport<W: Write>(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        match self {
            Action::PrevViewport => editor.viewports.prev_viewport(),
            Action::NextViewport => editor.viewports.next_viewport(),
            Action::DeleteViewport => {
                todo!()
            }
            Action::DeleteOtherViewport => {
                todo!()
            }

            _ => {}
        }

        Ok(())
    }
}
