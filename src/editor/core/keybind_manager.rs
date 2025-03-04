use core::fmt;
use std::{
    collections::{BTreeMap, HashMap},
    time::{Duration, Instant},
};

use super::{actions::action::Action, mode::Mode};
use crossterm::event::{KeyCode, KeyModifiers};
use mlua::{Lua, Table};

const LEADER: &str = "Space";
pub struct KeyAction {
    pub action: ActionOrClosure,
    pub desc: String,
}

impl KeyAction {
    pub fn new(action: ActionOrClosure, desc: String) -> Self {
        Self { action, desc }
    }
}
type Closure = dyn FnMut((&str, &(u16, u16))) -> Action;
pub enum ActionOrClosure {
    Static(Action),
    Dynamic(Box<Closure>),
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
///                          Mode,   Key,    Modifier
pub type Keybinds = HashMap<(String, String, String), KeyAction>;

type KeybindSort = Vec<((String, String), String)>;
type BtreeSort = BTreeMap<String, KeybindSort>;
pub struct KeybindManagerV2 {
    keybinds: Keybinds,
    last_pressed: Vec<(String, KeyCode, KeyModifiers, Instant)>,
    leader_pressed: bool,
    double_tap_threshold: Duration,
}

impl KeybindManagerV2 {
    pub fn new() -> Self {
        Self {
            keybinds: HashMap::new(),
            last_pressed: Vec::new(),
            leader_pressed: false,
            double_tap_threshold: Duration::from_millis(1000),
        }
    }

    pub fn load_keybinds_from_lua(&mut self) -> mlua::Result<()> {
        let lua = Lua::new();

        // Charger le fichier Lua
        let lua_code = std::fs::read_to_string("./config.lua").expect("cannot load lua file");
        let config: Table = lua.load(&lua_code).eval()?; // Charge le fichier Lua
        let keybinds_table: Table = config.get("keybinds")?; // Récupère la table "keybinds"

        for keybind_pair in keybinds_table.pairs::<String, Table>() {
            let (mode, mode_table) = keybind_pair?;
            for pair in mode_table.sequence_values::<Table>() {
                let action_table = pair?;
                let action = action_table.get::<String>("action")?;
                let desc = action_table.get::<String>("description")?;
                let modifiers = action_table.get::<String>("modifiers")?;
                let keys = action_table.get::<String>("key")?;

                self.bind_key(
                    mode.clone(),
                    keys.as_str(),
                    modifiers.as_str(),
                    KeyAction::new(ActionOrClosure::Static(Action::from(action)), desc),
                );
            }
        }

        Ok(())
    }

    pub fn load_dyn_keybinds(&mut self) {
        self.bind_key(
            Mode::Normal.to_string().to_lowercase(),
            "x",
            "",
            KeyAction::new(
                ActionOrClosure::Dynamic(Box::new(move |(_, v_cursor)| {
                    Action::RemoveCharAt(*v_cursor)
                })),
                "Deletes a character at a specific position.".to_string(),
            ),
        );
        self.bind_key(
            Mode::Command.to_string().to_lowercase(),
            "Return",
            "",
            KeyAction::new(
                ActionOrClosure::Dynamic(Box::new(move |(cmd, _): (&str, &(u16, u16))| {
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

    pub fn bind_key(&mut self, mode: String, keys: &str, modifiers: &str, action: KeyAction) {
        self.keybinds
            .insert((mode, keys.to_string(), modifiers.to_string()), action);
    }

    pub fn handle_keybinds(
        &mut self,
        mode: Mode,
        key: KeyCode,
        modifiers: KeyModifiers,
        v_cursor: &(u16, u16),
        cmd: &str,
        is_file_explorer: bool,
    ) -> Option<Action> {
        if mode == Mode::Normal && key.to_string() == LEADER {
            self.leader_pressed = true;
            return None;
        }

        let mode = match is_file_explorer {
            true => "file_explorer".to_string(),
            false => mode.to_string().to_lowercase(),
        };

        let action = match self.leader_pressed {
            true => self.handle_leader_keybinds(mode, key, modifiers, v_cursor, cmd),
            false => self.handle_normal_keybinds(mode, key, modifiers, v_cursor, cmd),
        };

        if action.is_some() {
            self.clear_input();
        }
        action
    }

    fn handle_leader_keybinds(
        &mut self,
        mode: String,
        key: KeyCode,
        modifiers: KeyModifiers,
        v_cursor: &(u16, u16),
        cmd: &str,
    ) -> Option<Action> {
        let sequence = format!("<leader>{key}");
        let modifier = match modifiers.is_empty() {
            true => "".to_string(),
            false => modifiers.to_string(),
        };

        match self
            .keybinds
            .get_mut(&(mode.clone(), sequence.clone(), modifier))
        {
            Some(key_action) => match &mut key_action.action {
                ActionOrClosure::Static(action) => Some(action.clone()),
                ActionOrClosure::Dynamic(closure) => Some(closure((cmd, v_cursor))),
            },
            None => self.handle_multiple_press(mode, key, modifiers, v_cursor, cmd),
        }
    }

    fn contains_keybinds(&self, seq: &String) -> bool {
        self.keybinds
            .iter()
            .any(|((_, key, _), _)| key.contains(seq))
    }

    fn handle_normal_keybinds(
        &mut self,
        mode: String,
        key: KeyCode,
        modifiers: KeyModifiers,
        v_cursor: &(u16, u16),
        cmd: &str,
    ) -> Option<Action> {
        let modifier = match modifiers.is_empty() {
            true => "".to_string(),
            false => modifiers.to_string(),
        };

        match self
            .keybinds
            .get_mut(&(mode.clone(), key.to_string(), modifier))
        {
            Some(key_action) => match &mut key_action.action {
                ActionOrClosure::Static(action) => Some(action.clone()),
                ActionOrClosure::Dynamic(closure) => Some(closure((cmd, v_cursor))),
            },
            None => {
                if let KeyCode::Char(c) = key {
                    let action = match mode.as_str() {
                        "command" => Some(Action::AddCommandChar(c)),
                        "search" => Some(Action::AddSearchChar(c)),
                        "insert" => Some(Action::AddChar(c)),
                        _ => None,
                    };
                    if action.is_some() {
                        return action;
                    }
                };
                self.handle_multiple_press(mode, key, modifiers, v_cursor, cmd)
            }
        }
    }

    fn handle_multiple_press(
        &mut self,
        mode: String,
        key: KeyCode,
        modifiers: KeyModifiers,
        v_cursor: &(u16, u16),
        cmd: &str,
    ) -> Option<Action> {
        let now = Instant::now();
        self.last_pressed.push((mode.clone(), key, modifiers, now));
        self.last_pressed
            .retain(|(_, _, _, time)| now.duration_since(*time) < self.double_tap_threshold);

        let mut sequence: String = self
            .last_pressed
            .iter()
            .filter(|(m, _, _, _)| *m == mode)
            .map(|(_, k, _, _)| k.to_string())
            .collect();

        if self.leader_pressed {
            sequence = format!("<leader>{sequence}");
        }
        let modifier = match modifiers.is_empty() {
            true => "".to_string(),
            false => modifiers.to_string(),
        };

        match self.keybinds.get_mut(&(mode, sequence.clone(), modifier)) {
            Some(key_action) => {
                let action = match &mut key_action.action {
                    ActionOrClosure::Static(action) => Some(action.clone()),
                    ActionOrClosure::Dynamic(closure) => Some(closure((cmd, v_cursor))),
                };
                return action;
            }
            None => {
                if !self.contains_keybinds(&sequence) && self.leader_pressed {
                    self.clear_input();
                }
            }
        };
        None
    }

    pub fn show_keybinds(&self) -> Vec<String> {
        let mut lines = vec![
            "--- For specific keybinds you can type ---".to_string(),
            "".to_string(),
            "map e / explorer      n / normal      c / command".to_string(),
            "map i / insert        e / visual      s / search".to_string(),
            "".to_string(),
        ];

        for (mode, vec) in self.sort_by_mode() {
            lines.push("".to_string());
            lines.push("".to_string());
            lines.push(format!("---- {mode} ----"));
            lines.push("".to_string());

            for ((key, modifier), desc) in vec {
                match modifier.is_empty() {
                    true => lines.push(format!("{key}   : {desc}")),
                    false => lines.push(format!("{modifier} {key}   : {desc}")),
                }
            }
        }

        lines
    }
    fn sort_by_mode(&self) -> BtreeSort {
        let mut sorted = BTreeMap::new();
        for ((mode, key, modifiers), action) in &self.keybinds {
            sorted
                .entry(mode.clone())
                .or_insert_with(Vec::new)
                .push(((key.clone(), modifiers.clone()), action.desc.clone()));
        }

        sorted
    }

    pub fn show_specific_keybinds(&self, mode: &str) -> Vec<String> {
        let mut lines = vec![format!("---- Keybind for {mode} ----"), "".to_string()];
        let lowercase_mode = mode.to_lowercase();
        let mode = match lowercase_mode.as_str() {
            "e" | "explorer" => "file_explorer",
            "n" => "normal",
            "c" => "command",
            "i" => "insert",
            "v" => "visual",
            "s" => "search",
            m => m,
        };

        for ((_, key, modifier), action) in self.keybinds.iter().filter(|k| k.0 .0 == mode) {
            match modifier.is_empty() {
                true => lines.push(format!("{key}   : {}", action.desc)),
                false => lines.push(format!("{modifier} {key}   : {}", action.desc)),
            }
        }
        lines
    }
}
