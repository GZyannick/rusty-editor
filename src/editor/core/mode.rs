use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
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
