use crate::log_message;

use super::action::Action;

pub struct Command;

impl Command {
    pub fn execute(command: &str) -> Option<Action> {
        match command {
            "w" => Some(Action::SaveFile),
            _ => {
                log_message!("{command}");
                None
            }
        }
    }
}
