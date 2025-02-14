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

impl PartialEq for OldCursorPosition {
    fn eq(&self, other: &Self) -> bool {
        self.cursor == other.cursor && self.top == other.top
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

impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::EnterMode(l0), Self::EnterMode(r0)) => l0 == r0,
            (Self::AddChar(l0), Self::AddChar(r0)) => l0 == r0,
            (Self::RemoveCharAt(l0), Self::RemoveCharAt(r0)) => l0 == r0,
            (Self::RemoveCharFrom(l0), Self::RemoveCharFrom(r0)) => l0 == r0,
            (Self::WaitingCmd(l0), Self::WaitingCmd(r0)) => l0 == r0,
            (Self::AddCommandChar(l0), Self::AddCommandChar(r0)) => l0 == r0,
            (Self::CreateFileOrDirectory(l0), Self::CreateFileOrDirectory(r0)) => l0 == r0,
            (Self::UndoDeleteLine(l0, l1), Self::UndoDeleteLine(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::UndoDeleteBlock(l0, l1), Self::UndoDeleteBlock(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::UndoNewLine(l0), Self::UndoNewLine(r0)) => l0 == r0,
            (Self::UndoMultiple(l0), Self::UndoMultiple(r0)) => l0 == r0,
            (Self::UndoCharAt(l0, l1), Self::UndoCharAt(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::UndoPast(l0, l1, l2), Self::UndoPast(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
            }
            (Self::AddSearchChar(l0), Self::AddSearchChar(r0)) => l0 == r0,
            (Self::GotoPos(l0), Self::GotoPos(r0)) => l0 == r0,
            (Self::UndoNewLineWithText(l0, l1), Self::UndoNewLineWithText(r0, r1)) => {
                l0 == r0 && l1 == r1
            }
            (Self::AddStr(l0), Self::AddStr(r0)) => l0 == r0,
            (Self::UndoStrAt(l0, l1, l2), Self::UndoStrAt(r0, r1, r2)) => {
                l0 == r0 && l1 == r1 && l2 == r2
            }
            (Self::RenameFileOrDirectory(l0), Self::RenameFileOrDirectory(r0)) => l0 == r0,
            (Self::AddModalChar(l0), Self::AddModalChar(r0)) => l0 == r0,
            (Self::HelpKeybinds(l0), Self::HelpKeybinds(r0)) => l0 == r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}
