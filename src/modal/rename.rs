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

impl ModalContent for ModalRenameFD {
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
            KeyCode::Esc => Some(Action::LeaveModal),
            KeyCode::Backspace => Some(Action::RemoveModalChar),
            KeyCode::Char(c) => Some(Action::AddModalChar(*c)),
            KeyCode::Enter => Some(Action::RenameFileOrDirectory(self.content.clone())),
            _ => None,
        };
        Ok(action)
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
}
