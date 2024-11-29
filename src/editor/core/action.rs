use super::mode::Mode;

pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterMode(Mode),
    AddChar(char),
    RemoveChar,
    AddCommandChar(char),
    NewLine,
    SaveFile,
    Quit,
} 
