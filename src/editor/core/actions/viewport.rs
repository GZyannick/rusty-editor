use std::io::Write;

use super::action::Action;
use crate::{editor::Editor, log_message};

impl Action {
    pub fn viewport<W: Write>(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        match self {
            Action::PrevViewport => {
                log_message!("prev_viewport");
            }
            Action::NextViewport => {
                log_message!("prev_viewport");
            }
            _ => {}
        }

        Ok(())
    }
}
