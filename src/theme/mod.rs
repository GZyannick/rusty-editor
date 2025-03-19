pub mod color_highligther;
pub mod icon;
use mlua::{self, Lua, Table};

use crate::{helper::lua_handler::get_home_file, log_message};
#[derive(Debug)]
pub struct Theme {
    pub bg0: (u8, u8, u8),
    pub fg0: (u8, u8, u8),
    pub default: (u8, u8, u8),
    pub cursor: (u8, u8, u8),
    pub bg1: (u8, u8, u8),
    pub gray: (u8, u8, u8),
    pub bright_red: (u8, u8, u8),
    pub bright_green: (u8, u8, u8),
    pub bright_yellow: (u8, u8, u8),
    pub bright_blue: (u8, u8, u8),
    pub bright_purple: (u8, u8, u8),
    pub bright_aqua: (u8, u8, u8),
    pub bright_orange: (u8, u8, u8),
    pub neutral_red: (u8, u8, u8),
    pub neutral_green: (u8, u8, u8),
    pub neutral_yellow: (u8, u8, u8),
    pub neutral_aqua: (u8, u8, u8),
    pub faded_purple: (u8, u8, u8),
    pub light_gray: (u8, u8, u8),
}

impl Theme {
    // we pass lua on both like that there is only one Lua::new
    fn get_default_theme(theme_name: &str, lua: &Lua) -> Table {
        let default_table = lua
            .load(include_str!("./colors.lua"))
            .eval::<Table>()
            .unwrap();

        default_table
            .get::<Table>(theme_name)
            .unwrap_or(default_table.get("default").unwrap())
    }

    // we check if there is a theme file and if it contains the named theme in config.lua
    // else we check in app_theme
    fn get_user_theme(theme_name: &str, lua: &Lua) -> Table {
        // we load the created user theme
        // we wanna be sure the app doenst panic so in each case we want a default theme
        match get_home_file(".rusty/themes.lua").unwrap_or(None) {
            Some(user_theme) => {
                let user_table: Table = match lua.load(&user_theme).eval() {
                    Ok(table) => table,
                    Err(_) => return Self::get_default_theme(theme_name, lua),
                };
                match user_table.get::<Table>(theme_name) {
                    Ok(theme) => theme,
                    Err(_) => Self::get_default_theme(theme_name, lua),
                }
            }
            None => Self::get_default_theme(theme_name, lua),
        }
    }

    fn get_theme_name(lua: &Lua) -> String {
        // we retrieve the name of the theme ( user theme or app theme)
        // we wanna be sure that the app doesnt panic! and use the default theme
        get_home_file(".rusty/config.lua")
            .unwrap_or(None)
            .and_then(|lua_code| lua.load(lua_code).eval::<Table>().ok())
            .and_then(|config| config.get::<String>("theme").ok())
            .unwrap_or_else(|| "default".to_string())
    }

    pub fn load_theme() -> mlua::Result<Self> {
        let lua = Lua::new();

        let theme_name = Self::get_theme_name(&lua);
        let theme = Self::get_user_theme(&theme_name, &lua);

        Self::from_lua_table(&theme)
    }

    fn get_color(table: &Table, key: &str) -> mlua::Result<(u8, u8, u8)> {
        let color: Table = table.get(key)?;
        Ok((color.get(1)?, color.get(2)?, color.get(3)?))
    }

    fn from_lua_table(table: &Table) -> mlua::Result<Self> {
        Ok(Self {
            bg0: Self::get_color(table, "bg0")?,
            fg0: Self::get_color(table, "fg0")?,
            default: Self::get_color(table, "default")?,
            cursor: Self::get_color(table, "cursor")?,
            bg1: Self::get_color(table, "bg1")?,
            gray: Self::get_color(table, "gray")?,
            bright_red: Self::get_color(table, "bright_red")?,
            bright_green: Self::get_color(table, "bright_green")?,
            bright_yellow: Self::get_color(table, "bright_yellow")?,
            bright_blue: Self::get_color(table, "bright_blue")?,
            bright_purple: Self::get_color(table, "bright_purple")?,
            bright_aqua: Self::get_color(table, "bright_aqua")?,
            bright_orange: Self::get_color(table, "bright_orange")?,
            neutral_red: Self::get_color(table, "neutral_red")?,
            neutral_green: Self::get_color(table, "neutral_green")?,
            neutral_yellow: Self::get_color(table, "neutral_yellow")?,
            neutral_aqua: Self::get_color(table, "neutral_aqua")?,
            faded_purple: Self::get_color(table, "faded_purple")?,
            light_gray: Self::get_color(table, "light_gray")?,
        })
    }
}
