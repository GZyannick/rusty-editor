use std::fmt::Debug;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::editor::core::actions::action::Action;

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
    fn basic_action(&self, code: &KeyCode) -> Option<Action> {
        // with calling basic action in handle action at the end with action.is_some
        // we can override some basic_action
        match code {
            KeyCode::Esc => Some(Action::LeaveModal),
            KeyCode::Backspace => Some(Action::RemoveModalChar),
            KeyCode::Char(c) => Some(Action::AddModalChar(*c)),
            _ => None,
        }
    }
}
