use std::fmt::Display;

pub enum Mode {
    Normal,
    Insert,
    Command,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Command => write!(f, "COMMAND"),
            Mode::Insert => write!(f, "INSERT"),
            Mode::Normal => write!(f, "NORMAL"),
        }
    }
}
