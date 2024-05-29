use mlua::Lua;
use mlua::prelude::{LuaFunction, LuaTable, LuaResult, LuaNil};

pub fn main(lua: &Lua, loaded: &LuaTable) -> LuaResult<()> {
    let name = "pretty";
    let src = r#"pretty = { _version = "0.0.1"}

function pretty.tprint (tbl, indent)
  if not indent then indent = 0 end
  local output = ""
  for k, v in pairs(tbl) do
    local formatting = string.rep(" ", indent) .. k .. ": "
    if type(v) == "table" then
      output = output .. formatting .. "\n"
      output = output .. pretty.tprint(v, indent+1)
    elseif type(v) == 'boolean' then
      output = output .. formatting .. tostring(v) .. "\n"
    else
      output = output .. formatting .. v .. "\n"
    end
  end
  return output
end"#;
    lua.load(src).exec()?;
    let module: LuaTable = lua.globals().get(name)?;
    loaded.set(name, module)?;
    lua.globals().set(name, LuaNil)?;

    Ok(())
}