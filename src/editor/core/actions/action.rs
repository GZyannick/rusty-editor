use crate::editor::{core::mode::Mode, CursorBlock};

#[derive(Debug, Clone)]
pub struct OldCursorPosition {
    pub cursor: (u16, u16),
    pub top: u16,
}

impl OldCursorPosition {
    pub fn new(cursor: (u16, u16), top: u16) -> Self {
        OldCursorPosition { cursor, top }
    }
}

#[derive(Debug, Clone)]
pub enum Action {
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    EnterMode(Mode),
    AddChar(char),
    RemoveChar,
    RemoveCharAt((u16, u16)),
    RemoveCharFrom(bool),
    WaitingCmd(char),
    DeleteLine,
    DeleteWord,
    AddCommandChar(char),
    NewLine,
    PageDown,
    PageUp,
    EndOfLine,
    StartOfLine,
    Save,
    CreateFileOrDirectory(String),
    EndOfFile,
    StartOfFile,
    CenterLine,
    Undo,
    Quit,
    ForceQuit,
    NewLineInsertionBelowCursor,
    NewLineInsertionAtCursor,
    UndoDeleteLine(OldCursorPosition, Option<String>), //cursor.1 , top, content
    UndoDeleteBlock(OldCursorPosition, Vec<Option<String>>), //cursor.1 , top, content
    UndoNewLine(OldCursorPosition),
    UndoMultiple(Vec<Action>),
    UndoCharAt(OldCursorPosition, (u16, u16)),
    ExecuteCommand,
    EnterFileOrDirectory,
    SwapViewportToExplorer,
    SwapViewportToPopupExplorer,
    DeleteBlock,
    YankBlock,
    Past,
    UndoPast(CursorBlock, u16, bool),
    YankLine,
    MovePrev,
    MoveNext,
    ClearToNormalMode,
    AddSearchChar(char),
    FindSearchValue,
    GotoPos((u16, u16)),
    IterNextSearch,
    UndoNewLineWithText(OldCursorPosition, usize),
    GotoParentDirectory,
    AddStr(String),
    UndoStrAt(OldCursorPosition, (u16, u16), usize),
    RenameFileOrDirectory(String),
    DeleteFileOrDirectory,
    LeaveModal,
    AddModalChar(char),
    RemoveModalChar,
    CreateInputModal,
    RenameInputModal,
    DeleteInputModal,
    HelpKeybinds(Option<String>),
}
