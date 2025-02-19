use core::fmt;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::log_message;

use super::{actions::action::Action, mode::Mode};
use crossterm::event::KeyCode;
use mlua::{Lua, Table};

const LEADER: &str = "Space";
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
pub enum ActionOrClosure {
    Static(Action),
    Dynamic(Box<dyn FnMut((&str, &(u16, u16))) -> Action>),
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

pub type Keybinds = HashMap<(Mode, String), KeyAction>;
pub struct KeybindManagerV2 {
    keybinds: Keybinds,
    last_pressed: Vec<(Mode, KeyCode, Instant)>,
    leader_pressed: bool,
    double_tap_threshold: Duration,
}

impl KeybindManagerV2 {
    pub fn new() -> Self {
        Self {
            keybinds: HashMap::new(),
            last_pressed: Vec::new(),
            leader_pressed: false,
            double_tap_threshold: Duration::from_millis(500),
        }
    }

    pub fn load_keybinds_from_lua(&mut self) -> mlua::Result<()> {
        let lua = Lua::new();

        // Charger le fichier Lua
        let lua_code = std::fs::read_to_string("./config.lua").expect("cannot load lua file");
        let config: Table = lua.load(&lua_code).eval()?; // Charge le fichier Lua
        let keybinds_table: Table = config.get("keybinds")?; // Récupère la table "keybinds"
                                                             // log_message!("--- KEYBIND TABLE ---\n{:#?}", keybinds_table);

        for keybind_pair in keybinds_table.pairs::<String, Table>() {
            let (mode, mode_table) = keybind_pair?;
            let mode = Mode::from(mode);
            for pair in mode_table.sequence_values::<Table>() {
                let action_table = pair?;
                let action = action_table.get::<String>("action")?;
                let desc = action_table.get::<String>("description")?;
                let keys = action_table.get::<String>("key")?;

                self.bind_key(
                    mode,
                    keys.as_str(),
                    KeyAction::new(ActionOrClosure::Static(Action::from(action)), desc),
                );
            }
        }

        Ok(())
    }

    pub fn load_dyn_keybinds(&mut self) {
        self.bind_key(
            Mode::Command,
            "Return",
            KeyAction::new(
                ActionOrClosure::Dynamic(Box::new(move |(cmd, v_cursor): (&str, &(u16, u16))| {
                    if cmd == "q" {
                        return Action::Quit;
                    } else if cmd == "q!" {
                        return Action::ForceQuit;
                    }
                    Action::ExecuteCommand
                })),
                "Executes the entered command.".to_string(),
            ),
        );
    }

    pub fn init_keybinds(&mut self) {
        self.load_keybinds_from_lua()
            .expect("Failed to load keybinds from lua");
        self.load_dyn_keybinds();
    }

    fn clear_input(&mut self) {
        self.leader_pressed = false;
        self.last_pressed = Vec::new();
    }

    pub fn bind_key(&mut self, mode: Mode, keys: &str, action: KeyAction) {
        self.keybinds.insert((mode, keys.to_string()), action);
    }

    pub fn handle_keybinds(
        &mut self,
        mode: Mode,
        key: KeyCode,
        v_cursor: &(u16, u16),
        cmd: &str,
    ) -> Option<Action> {
        if mode == Mode::Normal && key.to_string() == LEADER {
            self.leader_pressed = true;
            return None;
        }

        let action = match self.leader_pressed {
            true => self.handle_leader_keybinds(mode, key, v_cursor, cmd),
            false => self.handle_normal_keybinds(mode, key, v_cursor, cmd),
        };
        if action.is_some() {
            self.clear_input();
        }
        action
    }

    fn handle_leader_keybinds(
        &mut self,
        mode: Mode,
        key: KeyCode,
        v_cursor: &(u16, u16),
        cmd: &str,
    ) -> Option<Action> {
        let sequence = format!("<leader>{key}");

        match self.keybinds.get_mut(&(mode, sequence)) {
            Some(key_action) => match &mut key_action.action {
                ActionOrClosure::Static(action) => Some(action.clone()),
                ActionOrClosure::Dynamic(closure) => Some(closure((cmd, v_cursor))),
            },
            None => self.handle_multiple_press(mode, key, v_cursor, cmd),
        }
    }

    fn handle_normal_keybinds(
        &mut self,
        mode: Mode,
        key: KeyCode,
        v_cursor: &(u16, u16),
        cmd: &str,
    ) -> Option<Action> {
        match self.keybinds.get_mut(&(mode, key.to_string())) {
            Some(key_action) => match &mut key_action.action {
                ActionOrClosure::Static(action) => Some(action.clone()),
                ActionOrClosure::Dynamic(closure) => Some(closure((cmd, v_cursor))),
            },
            None => {
                if let KeyCode::Char(c) = key {
                    let action = match mode {
                        Mode::Command => Some(Action::AddCommandChar(c)),
                        Mode::Search => Some(Action::AddSearchChar(c)),
                        Mode::Insert => Some(Action::AddChar(c)),
                        _ => None,
                    };
                    if action.is_some() {
                        return action;
                    }
                };
                self.handle_multiple_press(mode, key, v_cursor, cmd)
            }
        }
    }

    fn handle_multiple_press(
        &mut self,
        mode: Mode,
        key: KeyCode,
        v_cursor: &(u16, u16),
        cmd: &str,
    ) -> Option<Action> {
        log_message!("this is handle_multiple_press");
        let now = Instant::now();
        self.last_pressed.push((mode, key, now));
        self.last_pressed
            .retain(|(_, _, time)| now.duration_since(*time) < self.double_tap_threshold);

        let mut sequence: String = self
            .last_pressed
            .iter()
            .filter(|(m, _, _)| *m == mode)
            .map(|(_, k, _)| k.to_string())
            .collect();

        if self.leader_pressed {
            sequence = format!("<leader>{sequence}");
        }

        if let Some(key_action) = self.keybinds.get_mut(&(mode, sequence)) {
            let action = match &mut key_action.action {
                ActionOrClosure::Static(action) => Some(action.clone()),
                ActionOrClosure::Dynamic(closure) => Some(closure((cmd, v_cursor))),
            };
            return action;
        }
        None
    }
}
