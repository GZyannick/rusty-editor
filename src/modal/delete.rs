use crossterm::event::{KeyCode, KeyModifiers};

use crate::editor::core::actions::action::Action;

use super::modal_trait::ModalContent;

#[derive(Debug)]
pub struct ModalDeleteFD {
    title: String,
    content: String,
}

impl ModalDeleteFD {
    pub fn new(title: String) -> Self {
        Self {
            title,
            content: String::new(),
        }
    }
}

impl ModalContent for ModalDeleteFD {
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
            KeyCode::Enter => {
                let content = self.content.to_lowercase();
                match content {
                    c if c == "y" || c == "yes" => Some(Action::DeleteFileOrDirectory),
                    _ => Some(Action::LeaveModal),
                }
            }
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
