use core::fmt;
use std::{collections::HashMap, mem};

use crossterm::event::{KeyCode, KeyModifiers};

use crate::editor::Editor;

use super::{actions::action::Action, mode::Mode};

pub enum ActionOrClosure {
    Static(Action),
    Dynamic(Box<dyn FnMut(&mut Editor) -> Action>),
}
impl fmt::Debug for ActionOrClosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionOrClosure::Static(action) => {
                write!(f, "Static({:?})", action)
            }
            ActionOrClosure::Dynamic(_) => {
                // Here you just print a placeholder because you cannot directly debug a closure
                write!(f, "Dynamic(<closure>)")
            }
        }
    }
}

#[derive(Debug)]
pub struct KeyAction {
    pub action: ActionOrClosure,
    pub desc: String,
}

impl KeyAction {
    pub fn new(action: ActionOrClosure, desc: String) -> Self {
        Self { action, desc }
    }
}
pub type Keybinds = HashMap<(KeyCode, KeyModifiers), KeyAction>;

#[derive(Debug)]
pub struct KeybindManager {
    pub normal_mode: Keybinds,
    pub visual_mode: Keybinds,
    pub command_mode: Keybinds,
    pub search_mode: Keybinds,
    pub insert_mode: Keybinds,
    pub file_explorer: Keybinds,
}

impl KeybindManager {
    pub fn new() -> Self {
        Self {
            normal_mode: Self::init_normal_mode_keybind(),
            visual_mode: Self::init_visual_mode_keybind(),
            command_mode: Self::init_command_mode_keybind(),
            search_mode: Self::init_search_mode_keybind(),
            insert_mode: Self::init_insert_mode_keybind(),
            file_explorer: Self::init_file_explorer_keybind(),
        }
    }

    // allow us mem::take each keybinds by mode
    pub fn take_by_mode(&mut self, mode: &Mode, is_file_explorer: bool) -> Keybinds {
        let mode = match mode {
            Mode::Normal if is_file_explorer => &mut self.file_explorer,
            Mode::Normal => &mut self.normal_mode,
            Mode::Insert => &mut self.insert_mode,
            Mode::Command => &mut self.command_mode,
            Mode::Visual => &mut self.visual_mode,
            Mode::Search => &mut self.search_mode,
        };
        mem::take(mode)
    }

    pub fn push_by_mode(&mut self, mode: &Mode, keybinds: Keybinds, is_file_explorer: bool) {
        match mode {
            Mode::Normal if is_file_explorer => self.file_explorer = keybinds,
            Mode::Normal => self.normal_mode = keybinds,
            Mode::Insert => self.insert_mode = keybinds,
            Mode::Command => self.command_mode = keybinds,
            Mode::Visual => self.visual_mode = keybinds,
            Mode::Search => self.search_mode = keybinds,
        };
    }

    pub fn init_normal_mode_keybind() -> Keybinds {
        let mut keybinds = HashMap::new();

        // Static keybinds

        keybinds.insert(
            (KeyCode::Char('v'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EnterMode(Mode::Visual)),
                "Switches to Visual mode.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('z'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::WaitingCmd('z')),
                "Waits for a second key press to execute a complex command.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char(' '), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::WaitingCmd(' ')),
                "Waits for a second key press to execute a complex command.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('u'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::Undo),
                "Reverts the last performed action.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char(':'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EnterMode(Mode::Command)),
                "Switches to Command mode.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('p'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::Past),
                "Pastes previously copied text.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Esc, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::ClearToNormalMode),
                "Returns to Normal mode and clears the current state.".to_string(),
            ),
        );

        // Search Actions
        keybinds.insert(
            (KeyCode::Char('/'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EnterMode(Mode::Search)),
                "Switches to Search mode.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('n'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::IterNextSearch),
                "Jumps to the next search occurrence.".to_string(),
            ),
        );

        // Insert Actions
        keybinds.insert(
            (KeyCode::Char('i'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EnterMode(Mode::Insert)),
                "Switches to Insert mode.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char('a'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EnterMode(Mode::Insert)),
                "Switches to Insert mode.".to_string(),
            ),
        );

        // Delete Actions
        keybinds.insert(
            (KeyCode::Char('x'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Dynamic(Box::new(move |editor: &mut Editor| {
                    Action::RemoveCharAt(editor.v_cursor())
                })),
                "Deletes a character at a specific position.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char('d'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::WaitingCmd('d')),
                "Waits for a second key press to execute a complex command.".to_string(),
            ),
        );

        // Create line Actions
        keybinds.insert(
            (KeyCode::Char('o'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::NewLineInsertionBelowCursor),
                "Inserts a new line below the cursor and enters Insert mode.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char('O'), KeyModifiers::SHIFT),
            KeyAction::new(
                ActionOrClosure::Static(Action::NewLineInsertionAtCursor),
                "Inserts a new line at the cursor position and enters Insert mode.".to_string(),
            ),
        );

        // Yank Actions
        keybinds.insert(
            (KeyCode::Char('y'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::WaitingCmd('y')),
                "Waits for a second key press to execute a complex command.".to_string(),
            ),
        );

        // Movement Actions
        keybinds.insert(
            (KeyCode::PageUp, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::PageUp),
                "Scrolls up by one page.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::PageDown, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::PageDown),
                "Scrolls down by one page.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('G'), KeyModifiers::SHIFT),
            KeyAction::new(
                ActionOrClosure::Static(Action::EndOfFile),
                "Moves the cursor to the end of the file.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char('g'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::WaitingCmd('g')),
                "Waits for a second key press to execute a complex command.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('$'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EndOfFile),
                "Moves the cursor to the end of the file.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char('0'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::StartOfLine),
                "Moves the cursor to the beginning of the current line.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Home, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::StartOfLine),
                "Moves the cursor to the beginning of the current line.".to_string(),
            ),
        );

        // Movement with Modifiers
        keybinds.insert(
            (KeyCode::Char('f'), KeyModifiers::CONTROL),
            KeyAction::new(
                ActionOrClosure::Static(Action::PageDown),
                "Scrolls down by one page.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char('b'), KeyModifiers::CONTROL),
            KeyAction::new(
                ActionOrClosure::Static(Action::PageUp),
                "Scrolls up by one page.".to_string(),
            ),
        );

        // Return all keybinds
        keybinds
    }

    pub fn handle_keybind(
        hash: &mut Keybinds,
        code: KeyCode,
        modifiers: KeyModifiers,
        editor: &mut Editor,
    ) -> anyhow::Result<Option<Action>> {
        if let Some(key_action) = hash.get_mut(&(code, modifiers)) {
            let action = match &mut key_action.action {
                ActionOrClosure::Static(action) => Some(action.clone()),
                ActionOrClosure::Dynamic(closure) => Some(closure(editor)),
            };
            return Ok(action);
        }

        if let KeyCode::Char(c) = code {
            let action = match editor.mode {
                Mode::Command => Some(Action::AddCommandChar(c)),
                Mode::Search => Some(Action::AddSearchChar(c)),
                Mode::Insert => Some(Action::AddChar(c)),
                _ => None,
            };
            return Ok(action);
        };
        Ok(None)
    }

    fn init_command_mode_keybind() -> Keybinds {
        let mut keybinds = HashMap::new();
        // Movement with Modifiers
        keybinds.insert(
            (KeyCode::Enter, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EnterMode(Mode::Normal)),
                "Switches to Normal mode.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Esc, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::ClearToNormalMode),
                "Returns to Normal mode and clears the current state.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Backspace, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::RemoveCharFrom(false)),
                "Deletes a character based on context (e.g., search or command).".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Enter, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Dynamic(Box::new(move |editor: &mut Editor| {
                    if editor.command.as_str() == "q" {
                        return Action::Quit;
                    } else if editor.command.as_str() == "q!" {
                        return Action::ForceQuit;
                    }
                    Action::ExecuteCommand
                })),
                "Executes the entered command.".to_string(),
            ),
        );
        keybinds
    }

    fn init_file_explorer_keybind() -> Keybinds {
        let mut keybinds = HashMap::new();
        // Movement with Modifiers
        keybinds.insert(
            (KeyCode::Char(' '), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::WaitingCmd(' ')),
                "Waits for a second key press to execute a complex command.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Enter, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EnterFileOrDirectory),
                "Opens a file or enters a directory in the file explorer.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('-'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::GotoParentDirectory),
                "Moves up to the parent directory in the file explorer.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('d'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::DeleteInputModal),
                "Opens a dialog to confirm file or directory deletion.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('r'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::RenameInputModal),
                "Opens a dialog to rename a file or directory.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('a'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::CreateInputModal),
                "Opens a dialog to create a new file or directory.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('i'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::CreateInputModal),
                "Opens a dialog to create a new file or directory.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('G'), KeyModifiers::SHIFT),
            KeyAction::new(
                ActionOrClosure::Static(Action::EndOfFile),
                "Moves the cursor to the end of the file.".to_string(),
            ),
        );

        keybinds.insert(
            (KeyCode::Char('g'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::WaitingCmd('g')),
                "Waits for a second key press to execute a complex command.".to_string(),
            ),
        );
        keybinds
    }

    fn init_search_mode_keybind() -> Keybinds {
        let mut keybinds = HashMap::new();
        // Movement with Modifiers
        keybinds.insert(
            (KeyCode::Esc, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::ClearToNormalMode),
                "Returns to Normal mode and clears the current state.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Backspace, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::RemoveCharFrom(true)),
                "Deletes a character based on context (e.g., search or command).".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Enter, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EnterMode(Mode::Normal)),
                "Switches to Normal mode.".to_string(),
            ),
        );
        keybinds
    }

    fn init_insert_mode_keybind() -> Keybinds {
        let mut keybinds = HashMap::new();
        // Movement with Modifiers

        keybinds.insert(
            (KeyCode::Esc, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EnterMode(Mode::Normal)),
                "Switches to Normal mode.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Backspace, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::RemoveChar),
                "Deletes the character before the cursor.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Enter, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::NewLine),
                "Inserts a new line below the current line.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Tab, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::AddStr("  ".into())),
                "Adds a string of text at the cursor position.".to_string(),
            ),
        );
        keybinds
    }

    fn init_visual_mode_keybind() -> Keybinds {
        let mut keybinds = HashMap::new();
        // Movement with Modifiers

        // Mode Switching
        keybinds.insert(
            (KeyCode::Esc, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EnterMode(Mode::Normal)),
                "Switches to Normal mode.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char(':'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EnterMode(Mode::Command)),
                "Switches to Command mode.".to_string(),
            ),
        );

        // Actions
        keybinds.insert(
            (KeyCode::Char('d'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::DeleteBlock),
                "Deletes a selected block of text.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char('y'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::YankBlock),
                "Copies a selected block of text.".to_string(),
            ),
        );

        // Movement
        keybinds.insert(
            (KeyCode::PageUp, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::PageUp),
                "Scrolls up by one page.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::PageDown, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::PageDown),
                "Scrolls down by one page.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char('G'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EndOfFile),
                "Moves the cursor to the end of the file.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char('g'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::WaitingCmd('g')),
                "Waits for a second key press to execute a complex command.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char('$'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EndOfLine),
                "Moves the cursor to the end of the current line.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::End, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::EndOfLine),
                "Moves the cursor to the end of the current line.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Char('0'), KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::StartOfLine),
                "Moves the cursor to the beginning of the current line.".to_string(),
            ),
        );
        keybinds.insert(
            (KeyCode::Home, KeyModifiers::empty()),
            KeyAction::new(
                ActionOrClosure::Static(Action::StartOfLine),
                "Moves the cursor to the beginning of the current line.".to_string(),
            ),
        );
        keybinds
    }

    pub fn show_keybinds(&self) -> Vec<String> {
        let mut lines: Vec<String> = vec![];
        let chain_keybinds = self
            .normal_mode
            .iter()
            .chain(self.visual_mode.iter())
            .chain(self.command_mode.iter())
            .chain(self.search_mode.iter())
            .chain(self.insert_mode.iter())
            .chain(self.file_explorer.iter());

        lines.push("--- For specific keybinds you can type ---".to_string());
        lines.push("".to_string());
        lines.push("map e / explorer      n / normal      c / command".to_string());
        lines.push("map i / insert        e / visual      s / search".to_string());
        lines.push("".to_string());
        lines.push("".to_string());

        for (k, v) in chain_keybinds {
            let key = match k.1 != KeyModifiers::empty() {
                true => format!("{} {}: {}", k.1, k.0, v.desc),
                false => format!("{}: {}", k.0, v.desc),
            };
            lines.push(key);
            lines.push(String::new());
        }
        lines
    }

    pub fn specific_keybinds(&self, mode: &str) -> Vec<String> {
        let mode = match mode.to_lowercase().as_str() {
            "explorer" | "e" => Some((&self.file_explorer, "File Explorer Keybinds")),
            "normal" | "n" => Some((&self.normal_mode, "Normal Keybinds")),
            "command" | "c" => Some((&self.command_mode, "Command Keybinds")),
            "insert" | "i" => Some((&self.insert_mode, "Insert Keybinds")),
            "visual" | "v" => Some((&self.visual_mode, "Visual Keybinds")),
            "search" | "s" => Some((&self.search_mode, "Search Keybinds")),
            _ => None,
        };
        if let Some((keybinds, name)) = mode {
            let mut lines: Vec<String> = vec![];
            lines.push(format!("----{name}----"));
            for (k, v) in keybinds {
                let key = match k.1 != KeyModifiers::empty() {
                    true => format!("{} {}: {}", k.1, k.0, v.desc),
                    false => format!("{}: {}", k.0, v.desc),
                };
                lines.push(key);
                lines.push(String::new());
            }
            lines
        } else {
            self.show_keybinds()
        }
    }
}
