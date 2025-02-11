use super::actions::action::Action;

pub struct Command;

impl Command {
    pub fn execute(command: &str) -> Option<Action> {
        match command {
            "w" => Some(Action::Save),
            "map" => Some(Action::HelpKeybinds(None)),
            cmd => {
                if let Ok(num) = cmd.parse::<u16>() {
                    return Some(Action::GotoPos((0, num)));
                }
                if cmd.contains("map") {
                    let cmd = cmd.replace("map", "");
                    return Some(Action::HelpKeybinds(Some(cmd.trim().to_string())));
                }
                None
            }
        }
    }
}
