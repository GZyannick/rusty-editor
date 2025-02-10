use crate::editor::Editor;
use anyhow::{Ok, Result};
use crossterm::QueueableCommand;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
};

use super::actions::action::Action;
use super::keybind_manager::KeybindManager;
use super::mode::Mode;

impl Editor {
    pub fn handle_action(&mut self, event: Event) -> Result<Option<Action>> {
        if let event::Event::Key(ev) = event {
            if ev.kind == event::KeyEventKind::Release {
                return Ok(None);
            }

            let code = ev.code;
            let modifiers = ev.modifiers;

            if let Some(ref mut modal) = self.modal {
                // this keybinds will still be related to handle_action
                // and not keybind manager because there are not global to the editor
                // but specific to each modal
                return modal.handle_action(&code, &modifiers);
            }

            if let Some(c) = self.waiting_command {
                let action = self.handle_waiting_command(c, &code);
                self.waiting_command = None;
                self.stdout
                    .queue(cursor::SetCursorStyle::DefaultUserShape)?;
                return Ok(action);
            }

            let nav = self.navigation(&code, &modifiers)?;
            if nav.is_some() {
                return Ok(nav);
            }

            return self.handle_keybinds(code, modifiers);
        }
        Ok(None)
    }

    fn handle_keybinds(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers, // not used for now
    ) -> Result<Option<Action>> {
        let is_file_explorer = self.viewports.c_viewport().is_file_explorer();
        let mut temp_keybinds = self.keybinds.take_by_mode(&self.mode, is_file_explorer);
        let result = KeybindManager::handle_keybind(&mut temp_keybinds, code, modifiers, self);
        self.keybinds
            .push_by_mode(&self.mode, temp_keybinds, is_file_explorer);
        result
    }

    fn handle_waiting_command(&mut self, c: char, code: &KeyCode) -> Option<Action> {
        match c {
            'd' => match code {
                KeyCode::Char('d') => Some(Action::DeleteLine),
                KeyCode::Char('w') => Some(Action::DeleteWord),
                _ => None,
            },
            'g' => match code {
                KeyCode::Char('g') => Some(Action::StartOfFile),
                _ => None,
            },
            'z' => match code {
                KeyCode::Char('z') => Some(Action::CenterLine),
                _ => None,
            },
            'y' => match code {
                KeyCode::Char('y') => Some(Action::YankLine),
                _ => None,
            },
            ' ' => match code {
                KeyCode::Char('e') => Some(Action::SwapViewportToPopupExplorer),
                KeyCode::Char('-') => Some(Action::SwapViewportToExplorer),
                _ => None,
            },
            _ => None,
        }
    }

    fn navigation(&mut self, code: &KeyCode, modifiers: &KeyModifiers) -> Result<Option<Action>> {
        let mut action: Option<Action> = None;

        if matches!(self.mode, Mode::Command) || matches!(self.mode, Mode::Search) {
            return Ok(action);
        }

        action = match code {
            KeyCode::Down => Some(Action::MoveDown),
            KeyCode::Up => Some(Action::MoveUp),
            KeyCode::Left => Some(Action::MoveLeft),
            KeyCode::Right => Some(Action::MoveRight),
            _ => None,
        };

        if !matches!(self.mode, Mode::Insert)
            && action.is_none()
            && matches!(modifiers, &KeyModifiers::NONE)
        {
            action = match code {
                KeyCode::Char('h') => Some(Action::MoveLeft),
                KeyCode::Char('j') => Some(Action::MoveDown),
                KeyCode::Char('k') => Some(Action::MoveUp),
                KeyCode::Char('l') => Some(Action::MoveRight),
                KeyCode::Char('w') => Some(Action::MoveNext), // Move next until the char is not
                // the same type than before
                KeyCode::Char('b') => Some(Action::MovePrev), // Move prev until the char is not
                // the same type than before
                // exemple if char is a letter Move until char is diff from letter
                _ => None,
            }
        };

        Ok(action)
    }
}
