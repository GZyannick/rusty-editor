use std::fmt::Debug;

use crossterm::event::{KeyCode, KeyModifiers};

use super::core::actions::action::Action;

pub trait ModalContent: Debug {
    fn title(&self) -> &str;
    fn body(&self) -> &str;
    fn handle_action(
        &self,
        code: &KeyCode,
        modifiers: &KeyModifiers,
    ) -> anyhow::Result<Option<Action>>;
    fn push(&mut self, ch: char);
    fn pop(&mut self);
}

#[derive(Debug)]
pub struct ModalInput {
    title: String,
    content: String,
}

impl ModalInput {
    pub fn new(title: String) -> Self {
        Self {
            title,
            content: String::new(),
        }
    }
}

impl ModalContent for ModalInput {
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
            KeyCode::Enter => Some(Action::CreateFileOrDirectory(self.content.clone())),
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
