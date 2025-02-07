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
}

impl Action {
    pub fn desc(&self) -> &'static str {
        match self {
            Action::MoveUp => "Moves the cursor up.",
            Action::MoveDown => "Moves the cursor down.",
            Action::MoveLeft => "Moves the cursor left.",
            Action::MoveRight => "Moves the cursor right.",
            Action::EnterMode(mode) => match mode {
                Mode::Normal => "Switches to Normal mode.",
                Mode::Insert => "Switches to Insert mode.",
                Mode::Command => "Switches to Command mode.",
                Mode::Visual => "Switches to Visual mode.",
                Mode::Search => "Switches to Search mode.",
            },
            Action::AddChar(_) => "Adds a character at the cursor's current position.",
            Action::RemoveChar => "Deletes the character before the cursor.",
            Action::RemoveCharAt(_) => "Deletes a character at a specific position.",
            Action::RemoveCharFrom(_) => {
                "Deletes a character based on context (e.g., search or command)."
            }
            Action::WaitingCmd(_) => "Waits for a second key press to execute a complex command.",
            Action::DeleteLine => "Deletes the entire current line.",
            Action::DeleteWord => "Deletes the word under the cursor.",
            Action::AddCommandChar(_) => "Adds a character to the command line.",
            Action::NewLine => "Inserts a new line below the current line.",
            Action::PageDown => "Scrolls down by one page.",
            Action::PageUp => "Scrolls up by one page.",
            Action::EndOfLine => "Moves the cursor to the end of the current line.",
            Action::StartOfLine => "Moves the cursor to the beginning of the current line.",
            Action::Save => "Saves the current file.",
            Action::CreateFileOrDirectory(_) => "Creates a new file or directory.",
            Action::EndOfFile => "Moves the cursor to the end of the file.",
            Action::StartOfFile => "Moves the cursor to the beginning of the file.",
            Action::CenterLine => "Centers the current line on the screen.",
            Action::Undo => "Reverts the last performed action.",
            Action::Quit => "Exits the editor.",
            Action::ForceQuit => "Exits the editor without saving changes.",
            Action::NewLineInsertionBelowCursor => {
                "Inserts a new line below the cursor and enters Insert mode."
            }
            Action::NewLineInsertionAtCursor => {
                "Inserts a new line at the cursor position and enters Insert mode."
            }
            Action::UndoDeleteLine(_, _) => "Restores a previously deleted line.",
            Action::UndoDeleteBlock(_, _) => "Restores a previously deleted block of text.",
            Action::UndoNewLine(_) => "Reverts a newly inserted line.",
            Action::UndoMultiple(_) => "Reverts multiple actions at once.",
            Action::UndoCharAt(_, _) => "Restores a deleted character at a specific position.",
            Action::ExecuteCommand => "Executes the entered command.",
            Action::EnterFileOrDirectory => {
                "Opens a file or enters a directory in the file explorer."
            }
            Action::SwapViewportToExplorer => "Displays the file explorer.",
            Action::SwapViewportToPopupExplorer => "Displays the file explorer in popup mode.",
            Action::DeleteBlock => "Deletes a selected block of text.",
            Action::YankBlock => "Copies a selected block of text.",
            Action::Past => "Pastes previously copied text.",
            Action::UndoPast(_, _, _) => "Reverts a paste action.",
            Action::YankLine => "Copies the current line.",
            Action::MovePrev => "Moves the cursor to the previous word.",
            Action::MoveNext => "Moves the cursor to the next word.",
            Action::ClearToNormalMode => "Returns to Normal mode and clears the current state.",
            Action::AddSearchChar(_) => "Adds a character to the search input.",
            Action::FindSearchValue => "Searches for the entered value.",
            Action::GotoPos(_) => "Moves the cursor to a specific position.",
            Action::IterNextSearch => "Jumps to the next search occurrence.",
            Action::UndoNewLineWithText(_, _) => "Reverts a new line that contains text.",
            Action::GotoParentDirectory => "Moves up to the parent directory in the file explorer.",
            Action::AddStr(_) => "Adds a string of text at the cursor position.",
            Action::UndoStrAt(_, _, _) => "Reverts an added string of text.",
            Action::RenameFileOrDirectory(_) => "Renames a file or directory.",
            Action::DeleteFileOrDirectory => "Deletes a file or directory.",
            Action::LeaveModal => "Closes an active modal window.",
            Action::AddModalChar(_) => "Adds a character in a modal input.",
            Action::RemoveModalChar => "Removes a character from a modal input.",
            Action::CreateInputModal => "Opens a dialog to create a new file or directory.",
            Action::RenameInputModal => "Opens a dialog to rename a file or directory.",
            Action::DeleteInputModal => "Opens a dialog to confirm file or directory deletion.",
        }
    }
}
