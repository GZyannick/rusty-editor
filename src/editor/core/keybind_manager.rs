use core::fmt;
use std::collections::HashMap;

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
pub struct KeybindManager {
    pub normal_mode: HashMap<(KeyCode, KeyModifiers), ActionOrClosure>,
    pub visual_mode: HashMap<(KeyCode, KeyModifiers), ActionOrClosure>,
    pub command_mode: HashMap<(KeyCode, KeyModifiers), ActionOrClosure>,
    pub search_mode: HashMap<(KeyCode, KeyModifiers), ActionOrClosure>,
    pub insert_mode: HashMap<(KeyCode, KeyModifiers), ActionOrClosure>,
    pub file_explorer: HashMap<(KeyCode, KeyModifiers), ActionOrClosure>,
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

    pub fn init_normal_mode_keybind() -> HashMap<(KeyCode, KeyModifiers), ActionOrClosure> {
        let mut keybinds = HashMap::new();

        // Static keybinds
        keybinds.insert(
            (KeyCode::Char('v'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EnterMode(Mode::Visual)),
        );
        keybinds.insert(
            (KeyCode::Char('z'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::WaitingCmd('z')),
        );
        keybinds.insert(
            (KeyCode::Char(' '), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::WaitingCmd(' ')),
        );
        keybinds.insert(
            (KeyCode::Char('u'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::Undo),
        );
        keybinds.insert(
            (KeyCode::Char(':'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EnterMode(Mode::Command)),
        );
        keybinds.insert(
            (KeyCode::Char('p'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::Past),
        );
        keybinds.insert(
            (KeyCode::Esc, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::ClearToNormalMode),
        );

        // Search Actions
        keybinds.insert(
            (KeyCode::Char('/'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EnterMode(Mode::Search)),
        );
        keybinds.insert(
            (KeyCode::Char('n'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::IterNextSearch),
        );

        // Insert Actions
        keybinds.insert(
            (KeyCode::Char('i'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EnterMode(Mode::Insert)),
        );
        keybinds.insert(
            (KeyCode::Char('a'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EnterMode(Mode::Insert)),
        ); // TODO Move cursor

        // Delete Actions
        keybinds.insert(
            (KeyCode::Char('x'), KeyModifiers::empty()),
            ActionOrClosure::Dynamic(Box::new(move |editor: &mut Editor| {
                Action::RemoveCharAt(editor.v_cursor())
            })),
        );
        keybinds.insert(
            (KeyCode::Char('d'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::WaitingCmd('d')),
        );

        // Create Actions
        keybinds.insert(
            (KeyCode::Char('o'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::NewLineInsertionBelowCursor),
        );
        keybinds.insert(
            (KeyCode::Char('O'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::NewLineInsertionAtCursor),
        );

        // Yank Actions
        keybinds.insert(
            (KeyCode::Char('y'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::WaitingCmd('y')),
        );

        // Movement Actions
        keybinds.insert(
            (KeyCode::PageUp, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::PageUp),
        );
        keybinds.insert(
            (KeyCode::PageDown, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::PageDown),
        );
        keybinds.insert(
            (KeyCode::Char('G'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EndOfFile),
        );
        keybinds.insert(
            (KeyCode::Char('g'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::WaitingCmd('g')),
        );
        keybinds.insert(
            (KeyCode::Char('$'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EndOfLine),
        );
        keybinds.insert(
            (KeyCode::Char('0'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::StartOfLine),
        );
        keybinds.insert(
            (KeyCode::Home, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::StartOfLine),
        );

        // Movement with Modifiers
        keybinds.insert(
            (KeyCode::Char('f'), KeyModifiers::CONTROL),
            ActionOrClosure::Static(Action::PageDown),
        );
        keybinds.insert(
            (KeyCode::Char('b'), KeyModifiers::CONTROL),
            ActionOrClosure::Static(Action::PageUp),
        );

        // Return all keybinds
        keybinds
    }

    pub fn handle_keybind(
        hash: &mut HashMap<(KeyCode, KeyModifiers), ActionOrClosure>,
        code: KeyCode,
        modifiers: KeyModifiers,
        editor: &mut Editor,
    ) -> anyhow::Result<Option<Action>> {
        if let Some(action_or_closure) = hash.get_mut(&(code, modifiers)) {
            let action = match action_or_closure {
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

    fn init_command_mode_keybind() -> HashMap<(KeyCode, KeyModifiers), ActionOrClosure> {
        let mut keybinds = HashMap::new();
        // Movement with Modifiers
        keybinds.insert(
            (KeyCode::Esc, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EnterMode(Mode::Normal)),
        );
        keybinds.insert(
            (KeyCode::Backspace, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::RemoveCharFrom(false)),
        );
        keybinds.insert(
            (KeyCode::Enter, KeyModifiers::empty()),
            ActionOrClosure::Dynamic(Box::new(move |editor: &mut Editor| {
                if editor.command.as_str() == "q" {
                    return Action::Quit;
                } else if editor.command.as_str() == "q!" {
                    return Action::ForceQuit;
                }
                Action::ExecuteCommand
            })),
        );
        keybinds
    }

    fn init_file_explorer_keybind() -> HashMap<(KeyCode, KeyModifiers), ActionOrClosure> {
        let mut keybinds = HashMap::new();
        // Movement with Modifiers
        keybinds.insert(
            (KeyCode::Char(' '), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::WaitingCmd(' ')),
        );

        keybinds.insert(
            (KeyCode::Enter, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EnterFileOrDirectory),
        );

        keybinds.insert(
            (KeyCode::Char('-'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::GotoParentDirectory),
        );

        keybinds.insert(
            (KeyCode::Char('d'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::DeleteInputModal),
        );

        keybinds.insert(
            (KeyCode::Char('r'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::RenameInputModal),
        );

        keybinds.insert(
            (KeyCode::Char('a'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::CreateInputModal),
        );

        keybinds.insert(
            (KeyCode::Char('i'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::CreateInputModal),
        );

        keybinds.insert(
            (KeyCode::Char('G'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EndOfFile),
        );

        keybinds.insert(
            (KeyCode::Char('g'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::WaitingCmd('g')),
        );
        keybinds
    }

    fn init_search_mode_keybind() -> HashMap<(KeyCode, KeyModifiers), ActionOrClosure> {
        let mut keybinds = HashMap::new();
        // Movement with Modifiers
        keybinds.insert(
            (KeyCode::Esc, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::ClearToNormalMode),
        );
        keybinds.insert(
            (KeyCode::Backspace, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::RemoveCharFrom(true)),
        );
        keybinds.insert(
            (KeyCode::Enter, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EnterMode(Mode::Normal)),
        );
        keybinds
    }

    fn init_insert_mode_keybind() -> HashMap<(KeyCode, KeyModifiers), ActionOrClosure> {
        let mut keybinds = HashMap::new();
        // Movement with Modifiers

        keybinds.insert(
            (KeyCode::Esc, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EnterMode(Mode::Normal)),
        );
        keybinds.insert(
            (KeyCode::Backspace, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::RemoveChar),
        );
        keybinds.insert(
            (KeyCode::Enter, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::NewLine),
        );
        keybinds.insert(
            (KeyCode::Tab, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::AddStr("  ".into())),
        );
        keybinds
    }

    fn init_visual_mode_keybind() -> HashMap<(KeyCode, KeyModifiers), ActionOrClosure> {
        let mut keybinds = HashMap::new();
        // Movement with Modifiers

        // Mode Switching
        keybinds.insert(
            (KeyCode::Esc, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EnterMode(Mode::Normal)),
        );
        keybinds.insert(
            (KeyCode::Char(':'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EnterMode(Mode::Command)),
        );

        // Actions
        keybinds.insert(
            (KeyCode::Char('d'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::DeleteBlock),
        );
        keybinds.insert(
            (KeyCode::Char('y'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::YankBlock),
        );

        // Movement
        keybinds.insert(
            (KeyCode::PageUp, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::PageUp),
        );
        keybinds.insert(
            (KeyCode::PageDown, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::PageDown),
        );
        keybinds.insert(
            (KeyCode::Char('G'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EndOfFile),
        );
        keybinds.insert(
            (KeyCode::Char('g'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::WaitingCmd('g')),
        );
        keybinds.insert(
            (KeyCode::Char('$'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EndOfLine),
        );
        keybinds.insert(
            (KeyCode::End, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::EndOfLine),
        );
        keybinds.insert(
            (KeyCode::Char('0'), KeyModifiers::empty()),
            ActionOrClosure::Static(Action::StartOfLine),
        );
        keybinds.insert(
            (KeyCode::Home, KeyModifiers::empty()),
            ActionOrClosure::Static(Action::StartOfLine),
        );
        keybinds
    }
}
