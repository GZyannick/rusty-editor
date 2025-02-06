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
}
