use mlua::prelude::*;

pub fn preload_serde_json(lua: &Lua) -> LuaResult<()> {
    let serde_json_table = lua.create_table()?;
    serde_json_table.set("from_table", lua.create_function(|_, t: LuaTable| {
        let json_str = serde_json::to_string(&t).map_err(LuaError::external)?;
        Ok(json_str)
    })?)?;
    serde_json_table.set("to_table", lua.create_function(|l: &Lua, s: String| {
        let json_val: serde_json::Value = serde_json::from_str(&s).map_err(LuaError::external)?;
        let lua_val = l.to_value(&json_val)?;
        Ok(lua_val)
    })?)?;

    let package: LuaTable = lua.globals().get("package")?;
    let loaded: LuaTable = package.get("loaded")?;
    loaded.set("serde_json", serde_json_table)?;

    Ok(())
}

pub fn mock_non_deterministic_globals(lua: &Lua) -> LuaResult<()> {
    let globals: LuaTable = lua.globals();

    let math_table: LuaTable = globals.get("math")?;
    math_table.set("random", lua.create_function(|_, ()| Ok(0.5))?)?;

    Ok(())
}