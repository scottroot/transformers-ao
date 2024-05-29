use mlua::Lua;
use mlua::prelude::{LuaFunction, LuaTable, LuaResult, LuaNil};

pub fn main(lua: &Lua, loaded: &LuaTable) -> LuaResult<()> {
    let name = "default";
    let src = r#"-- default handler for aos
function default(insertInbox)
  return function (msg)
    -- Add Message to Inbox
    insertInbox(msg)

    local txt = Colors.gray .. "New Message From " .. Colors.green ..
    (msg.From and (msg.From:sub(1,3) .. "..." .. msg.From:sub(-3)) or "unknown") .. Colors.gray .. ": "
    if msg.Action then
      txt = txt .. Colors.gray .. (msg.Action and ("Action = " .. Colors.blue .. msg.Action:sub(1,20)) or "") .. Colors.reset
    else
      local data = msg.Data
      if type(data) == 'table' then
        data = require('json').encode(data)
      end
      txt = txt .. Colors.gray .. "Data = " .. Colors.blue .. (data and data:sub(1,20) or "") .. Colors.reset
    end
    -- Print to Output
    print(txt)
  end

end"#;
    lua.load(src).exec()?;
    let module: LuaFunction = lua.globals().get(name)?;
    loaded.set(name, module)?;
    lua.globals().set(name, LuaNil)?;

    Ok(())
}