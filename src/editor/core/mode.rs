use std::fmt::Display;

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    Visual,
    Search,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Command => write!(f, "COMMAND"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Visual => write!(f, "VISUAL"),
            Mode::Normal => write!(f, "NORMAL"),
            Mode::Search => write!(f, "SEARCH"),
        }
    }
}

impl From<String> for Mode {
    fn from(value: String) -> Self {
        let value = value.to_lowercase();
        let mode = match value.as_str() {
            "visual" => Mode::Visual,
            "insert" => Mode::Insert,
            "command" => Mode::Command,
            "normal" => Mode::Normal,
            "search" => Mode::Search,
            _ => panic!("couldnt find mode"),
        };
        mode
    }
}
