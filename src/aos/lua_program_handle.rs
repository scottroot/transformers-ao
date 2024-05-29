use mlua::Lua;
use mlua::prelude::{LuaFunction, LuaTable, LuaResult, LuaNil};

pub fn main(lua: &Lua) -> LuaResult<()> {
    // let name = "handle"; // loader.lua
    let src = r#"
local json = require "json"
local process = require "process"
ao = require "ao"

function handle(msgJSON, aoJSON)
    -- decode inputs
    local msg = json.decode(msgJSON)
    local env = json.decode(aoJSON)
    ao.init(env)
    -- relocate custom tags to root message
    msg = ao.normalize(msg)

    local status, response = pcall(function()
        return (process.handle(msg, ao))
    end)

    -- encode output
    local responseJSON = json.encode({ok = status, response = response})
    return responseJSON
end
"#;
    lua.load(src).exec()?;
    // let module: LuaTable = lua.globals().get(name)?;
    // let module: LuaTable = lua.globals().get(name)?;
    // loaded.set(name, module)?;
    // lua.globals().set(name, LuaNil)?;

    Ok(())
}