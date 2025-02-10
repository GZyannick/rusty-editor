use crate::log_message;

use super::actions::action::Action;

pub struct Command;

impl Command {
    pub fn execute(command: &str) -> Option<Action> {
        match command {
            "w" => Some(Action::Save),
            "help keybinds" => Some(Action::HelpKeybinds(None)),

            cmd => {
                if let Ok(num) = cmd.parse::<u16>() {
                    return Some(Action::GotoPos((0, num)));
                }
                if cmd.contains("help keybinds") {
                    let cmd = cmd.replace("help keybinds", "");
                    return Some(Action::HelpKeybinds(Some(cmd.trim().to_string())));
                }
                None
            }
        }
    }
}
