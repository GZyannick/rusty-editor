use std::io::Write;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::editor::{core::actions::action::Action, Editor};

use super::modal_trait::ModalContent;

#[derive(Debug)]
pub struct ModalCreateFD {
    title: String,
    content: String,
}

impl ModalCreateFD {
    pub fn new(title: String) -> Self {
        Self {
            title,
            content: String::new(),
        }
    }
}

impl<W: Write> ModalContent<W> for ModalCreateFD {
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
            KeyCode::Enter => Some(Action::CreateFileOrDirectory(self.content.clone())),
            _ => None,
        };
        match action.is_some() {
            true => Ok(action),
            false => Ok(<ModalCreateFD as ModalContent<W>>::basic_action(self, code)),
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

    fn draw_modal(&self, editor: &mut Editor<W>) -> anyhow::Result<()> {
        self.draw_default(editor)
    }
}
