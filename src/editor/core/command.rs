use super::actions::action::Action;

pub struct Command;

impl Command {
    pub fn execute(command: &str) -> Option<Action> {
        match command {
            "w" => Some(Action::SaveFile),

            cmd => {
                if let Ok(num) = cmd.parse::<u16>() {
                    return Some(Action::GotoPos((0, num)));
                }
                None
            }
        }
    }
}
