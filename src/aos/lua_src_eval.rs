use mlua::{Lua, Table as LuaTable, Result as LuaResult, Nil as LuaNil};
use mlua::prelude::LuaFunction;

pub fn main(lua: &Lua, loaded: &LuaTable) -> LuaResult<()> {
    let name = "eval";
    let src = r#"local stringify = require("stringify")
-- handler for eval
function eval(ao)
  return function (msg)
    -- exec expression
    local expr = msg.Data
    local func, err = load("return " .. expr, 'aos', 't', _G)
    local output = ""
    local e = nil
    if err then
      func, err = load(expr, 'aos', 't', _G)
    end
    if func then
      output, e = func()
    else
      ao.outbox.Error = err
      return
    end
    if e then
      ao.outbox.Error = e
      return
    end
    -- set result in outbox.Output
    ao.outbox.Output = { data = {
      json = type(output) == "table" and pcall(function () return json.encode(output) end) and output or "undefined",
      output = type(output) == "table" and stringify.format(output) or output,
      prompt = Prompt()
    }}
  end
end
"#;
    lua.load(src).exec()?;
    let module: LuaFunction = lua.globals().get(name)?;
    loaded.set(name, module)?;
    lua.globals().set(name, LuaNil)?;

    Ok(())
}