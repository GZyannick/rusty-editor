pub mod color_highligther;
pub mod icon;
use mlua::{self, Lua, Table};
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
    pub fn load_theme(theme_name: &str) -> mlua::Result<Self> {
        let lua = Lua::new();
        let theme_str = include_str!("./colors.lua").to_string();
        let theme_table: Table = lua.load(&theme_str).eval().unwrap();
        let theme: Table = theme_table
            .get(theme_name)
            .expect("Couldnt find theme name");

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
