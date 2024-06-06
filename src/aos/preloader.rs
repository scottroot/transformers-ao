use mlua::prelude::*;

pub fn set_loaded(lua: &Lua, name: &str, content: &str) -> LuaResult<()> {
    let package: LuaTable = lua.globals().get("package")?;
    let loaded: LuaTable = package.get("loaded")?;
    let value: LuaTable = lua.load(content).set_name(name).eval()?;
    loaded.set(name, value)?;
    Ok(())
}
pub fn set_bundle(lua: &Lua, name: &str, content: &str) -> LuaResult<()> {
    let bundle: LuaTable = lua.globals().get("lua_bundle")?;
    bundle.set(name, content)?;
    Ok(())
}

pub fn exec(lua: &Lua, name: &str, content: &str) -> LuaResult<()> {
    lua.load(content).set_name(name).exec()?;
    Ok(())
}



