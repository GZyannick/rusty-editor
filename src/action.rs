use  crate::mode::Mode;

pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterMode(Mode),
    AddChar(char),
    AddCommandChar(char),
    Quit,
}
