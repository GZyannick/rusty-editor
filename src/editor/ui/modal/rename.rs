use std::io::Write;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::editor::core::actions::action::Action;

use super::modal_trait::ModalContent;

#[derive(Debug)]
pub struct ModalRenameFD {
    title: String,
    content: String,
}

impl ModalRenameFD {
    pub fn new(title: String, content: String) -> Self {
        Self { title, content }
    }
}

impl<W: Write> ModalContent<W> for ModalRenameFD {
    fn title(&self) -> &str {
        &self.title
    }

    fn body(&self) -> &str {
        &self.content
    }

    fn handle_action(
        &self,
        code: &KeyCode,
        _modifiers: &KeyModifiers,
    ) -> anyhow::Result<Option<Action>> {
        let action = match code {
            KeyCode::Enter => Some(Action::RenameFileOrDirectory(self.content.clone())),
            _ => None,
        };
        match action.is_some() {
            true => Ok(action),
            false => Ok(<ModalRenameFD as ModalContent<W>>::basic_action(self, code)),
        }
    }

    fn push(&mut self, ch: char) {
        self.content.push(ch);
    }
    //
    fn pop(&mut self) {
        if !self.content.is_empty() {
            self.content.pop();
        }
    }
    fn draw_modal(&self, editor: &mut crate::editor::Editor<W>) -> anyhow::Result<()> {
        self.draw_default(editor)
    }
}
