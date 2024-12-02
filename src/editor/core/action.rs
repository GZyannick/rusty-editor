use super::mode::Mode;

pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterMode(Mode),
    AddChar(char),
    RemoveChar,
    RemoveCharCursorPosition,
    WaitingCmd(char),
    DeleteLine,
    DeleteWord,
    AddCommandChar(char),
    NewLine(bool), // the bool is to know if we create the new line with or without the text behind
    // the cursor like with Enter we want the text behind
    PageDown,
    PageUp,
    EndOfLine,
    StartOfLine,
    SaveFile,
    EndOfFile,
    StartOfFile,
    Quit,
}
