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

#[cfg(test)]
mod tests_command {
    use crate::editor::core::actions::action::Action;

    use super::Command;
    #[test]
    fn test_basic_command() {
        let result = Command::execute("w");
        assert!(result == Some(Action::Save), "w shoudl save the app")
    }

    #[test]
    fn test_command_with_param() {
        let result = Command::execute("map e");
        assert!(
            result == Some(Action::HelpKeybinds(Some("e".to_string()))),
            "help keybinds should have e in param"
        )
    }

    #[test]
    fn test_false_command() {
        let result = Command::execute("false_cmd");
        assert!(result.is_none(), "should be none")
    }
}
