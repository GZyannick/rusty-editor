use crate::editor::Editor;
use anyhow::{Ok, Result};
use crossterm::QueueableCommand;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
};

use super::mode::Mode;
use crate::editor::core::actions::Action;

impl Editor {
    pub fn handle_action(&mut self, event: Event) -> Result<Option<Action>> {
        if let event::Event::Key(ev) = event {
            if ev.kind == event::KeyEventKind::Release {
                return Ok(None);
            }

            let code = ev.code;
            let modifiers = ev.modifiers;

            if let Some(c) = self.waiting_command {
                let action = self.handle_waiting_command(c, &code);
                self.waiting_command = None;
                self.stdout
                    .queue(cursor::SetCursorStyle::DefaultUserShape)?;
                return Ok(action);
            }

            let nav = self.navigation(&code)?;
            if nav.is_some() {
                return Ok(nav);
            }

            return match self.mode {
                Mode::Normal => self.handle_normal_event(&code, &modifiers),
                Mode::Command => self.handle_command_event(&code, &modifiers),
                Mode::Insert => self.handle_insert_event(&code, &modifiers),
                Mode::Visual => self.handle_visual_event(&code, &modifiers),
                Mode::Search => self.handle_search_event(&code, &modifiers),
            };
        }
        Ok(None)
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

    fn handle_search_event(
        &mut self,
        code: &KeyCode,
        _modifiers: &KeyModifiers, // not used for now
    ) -> Result<Option<Action>> {
        let action = match code {
            KeyCode::Enter => Some(Action::EnterMode(Mode::Normal)),
            KeyCode::Esc => Some(Action::ClearToNormalMode),
            KeyCode::Char(c) => Some(Action::AddSearchChar(*c)),
            KeyCode::Backspace => Some(Action::RemoveCharFrom(true)),
            _ => None,
        };
        Ok(action)
    }

    fn handle_insert_event(
        &mut self,
        code: &KeyCode,
        _modifiers: &KeyModifiers, // not used for now
    ) -> Result<Option<Action>> {
        let action = match code {
            KeyCode::Esc => Some(Action::EnterMode(Mode::Normal)),
            KeyCode::Backspace => Some(Action::RemoveChar),
            KeyCode::Enter => Some(Action::NewLine),
            KeyCode::Char(c) => Some(Action::AddChar(*c)),
            _ => None,
        };

        Ok(action)
    }

    fn handle_visual_event(
        &mut self,
        code: &KeyCode,
        _modifiers: &KeyModifiers,
    ) -> Result<Option<Action>> {
        let action = match code {
            KeyCode::Esc => Some(Action::EnterMode(Mode::Normal)),
            KeyCode::Char(':') => Some(Action::EnterMode(Mode::Command)),

            // action
            KeyCode::Char('d') => Some(Action::DeleteBlock),
            KeyCode::Char('y') => Some(Action::YankBlock),

            // movement
            KeyCode::PageUp => Some(Action::PageUp),
            KeyCode::PageDown => Some(Action::PageDown),
            KeyCode::Char('G') => Some(Action::EndOfFile),
            KeyCode::Char('g') => Some(Action::WaitingCmd('g')),
            KeyCode::Char('$') | KeyCode::End => Some(Action::EndOfLine),
            KeyCode::Char('0') | KeyCode::Home => Some(Action::StartOfLine),
            _ => None,
        };

        Ok(action)
    }

    fn handle_normal_event(
        &mut self,
        code: &KeyCode,
        modifiers: &KeyModifiers,
    ) -> Result<Option<Action>> {
        let action = match code {
            KeyCode::Char('v') => Some(Action::EnterMode(Mode::Visual)),
            KeyCode::Char('z') => Some(Action::WaitingCmd('z')),
            KeyCode::Char(' ') => Some(Action::WaitingCmd(' ')),
            KeyCode::Char('u') => Some(Action::Undo),
            KeyCode::Char(':') => Some(Action::EnterMode(Mode::Command)),
            KeyCode::Char('p') => Some(Action::Past),
            KeyCode::Esc => Some(Action::ClearToNormalMode),

            // handle file_explorer viewport
            KeyCode::Enter if self.viewports.c_viewport().is_file_explorer() => {
                Some(Action::EnterFileOrDirectory)
            }

            // Search Action
            KeyCode::Char('/') => Some(Action::EnterMode(Mode::Search)),
            KeyCode::Char('n') => Some(Action::IterNextSearch),

            // Insert Action
            KeyCode::Char('i') => Some(Action::EnterMode(Mode::Insert)),
            KeyCode::Char('a') => Some(Action::EnterMode(Mode::Insert)), //TODO Move cursor to
            //cursor right 1 time

            // Delete Action
            KeyCode::Char('x') => Some(Action::RemoveCharAt(self.v_cursor())),
            KeyCode::Char('d') => Some(Action::WaitingCmd('d')),

            // Create Action
            KeyCode::Char('o') => Some(Action::NewLineInsertionBelowCursor),
            KeyCode::Char('O') => Some(Action::NewLineInsertionAtCursor),

            // Yank Action
            KeyCode::Char('y') => Some(Action::WaitingCmd('y')),

            //Movement Action
            KeyCode::PageUp => Some(Action::PageUp),
            KeyCode::PageDown => Some(Action::PageDown),
            KeyCode::Char('G') => Some(Action::EndOfFile),
            KeyCode::Char('g') => Some(Action::WaitingCmd('g')),
            KeyCode::Char('$') | KeyCode::End => Some(Action::EndOfLine),
            KeyCode::Char('0') | KeyCode::Home => Some(Action::StartOfLine),

            // Movement with Modifiers
            KeyCode::Char('f') if matches!(modifiers, &KeyModifiers::CONTROL) => {
                Some(Action::PageDown)
            }

            KeyCode::Char('b') if matches!(modifiers, &KeyModifiers::CONTROL) => {
                Some(Action::PageUp)
            }

            _ => None,
        };

        Ok(action)
    }

    fn handle_command_event(
        &mut self,
        code: &KeyCode,
        _modifiers: &KeyModifiers, // not used for now
    ) -> Result<Option<Action>> {
        let action = match code {
            KeyCode::Esc => Some(Action::EnterMode(Mode::Normal)),
            KeyCode::Char('w') => Some(Action::SaveFile),
            KeyCode::Char(c) => Some(Action::AddCommandChar(*c)),
            KeyCode::Enter => {
                // handle the quit here to break the loop
                if self.command.as_str() == "q" {
                    return Ok(Some(Action::Quit));
                }
                Some(Action::ExecuteCommand)
            }
            KeyCode::Backspace => Some(Action::RemoveCharFrom(false)),
            _ => None,
        };

        Ok(action)
    }

    fn navigation(&mut self, code: &KeyCode) -> Result<Option<Action>> {
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

        if !matches!(self.mode, Mode::Insert) && action.is_none() {
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
