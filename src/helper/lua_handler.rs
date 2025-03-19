pub fn get_home_file(path: &str) -> mlua::Result<Option<String>> {
    let config_path = dirs::home_dir().unwrap().join(&path);

    // if there is no config.lua skip this part
    if !std::path::Path::new(&config_path).exists() {
        return Ok(None);
    }
    let lua_code = std::fs::read_to_string(&config_path).expect("Couldnt load lua file");
    match lua_code.is_empty() {
        true => Ok(None),
        false => Ok(Some(lua_code)),
    }
}
