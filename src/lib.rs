mod aos;
use aos::preloader;

mod models;

use std::ffi::{c_char, CStr, CString};
use mlua::Lua;
use mlua::prelude::*;


fn to_c_string(rust_string: String) -> *const c_char {
    CString::new(rust_string).unwrap().into_raw()
}

fn boot_lua(lua: &mut Lua) -> LuaResult<()> {
    let globals = lua.globals();
    let _print: LuaFunction = lua.create_function(|_, s: String| {
        println!("{}", s);
        Ok(())
    })?;
    globals.set("println", _print)?;

    let _stringify: LuaFunction = lua.create_function(|_, t: LuaTable| {
        let json_str = serde_json::to_string(&t).map_err(LuaError::external)?;
        Ok(json_str)
    })?;
    globals.set("_stringify", _stringify)?;

    models::bert::preload(lua)?;

    globals.set("lua_bundle", lua.create_table()?)?;
    preloader::exec(lua, "main", include_str!("aos/main.lua"))?;

    preloader::set_loaded(lua, "ao", include_str!("aos/ao.lua"))?;
    preloader::set_loaded(lua, "json", include_str!("aos/json.lua"))?;

    preloader::set_bundle(lua, ".pretty", include_str!("aos/pretty.lua"))?;
    preloader::set_bundle(lua, ".base64", include_str!("aos/base64.lua"))?;
    preloader::set_bundle(lua, ".chance", include_str!("aos/chance.lua"))?;
    preloader::set_bundle(lua, ".dump", include_str!("aos/dump.lua"))?;
    preloader::set_bundle(lua, ".utils", include_str!("aos/utils.lua"))?;
    preloader::set_bundle(lua, ".handlers-utils", include_str!("aos/handlers-utils.lua"))?;
    preloader::set_bundle(lua, ".handlers", include_str!("aos/handlers.lua"))?;
    preloader::set_bundle(lua, ".stringify", include_str!("aos/stringify.lua"))?;
    preloader::set_bundle(lua, ".eval", include_str!("aos/eval.lua"))?;
    preloader::set_bundle(lua, ".default", include_str!("aos/default.lua"))?;
    preloader::set_bundle(lua, ".handlers", include_str!("aos/handlers.lua"))?;

    preloader::set_loaded(lua, ".process", include_str!("aos/process.lua"))?;
    preloader::exec(lua, "loader", include_str!("aos/loader.lua"))?;


    lua.load(r#"Handlers.add("pingpong", Handlers.utils.hasMatchingData("ping"), Handlers.utils.reply("pong"))"#).exec()?;

    Ok(())
}

#[no_mangle]
pub extern "C" fn handle(arg0: *const c_char, arg1: *const c_char) -> *const c_char {
    let arg0_str = unsafe {
        if arg0.is_null() {
            eprintln!("Handle arg0 is null");
            return to_c_string("".to_string());
        }
        match CStr::from_ptr(arg0).to_str() {
            Ok(s) => s,
            Err(err) => {
                eprintln!("Handle arg0 is invalid UTF-8\\n");
                eprintln!("{}", err);
                return to_c_string("".to_string())
            },
        }
    };
    let arg1_str = unsafe {
        if arg1.is_null() {
            eprintln!("Handle arg1 is null");
            return to_c_string("".to_string());
        }
        match CStr::from_ptr(arg1).to_str() {
            Ok(s) => s,
            Err(err) => {
                eprintln!("Handle arg1 is invalid UTF-8\\n");
                eprintln!("{}", err);
                return to_c_string("".to_string())
            },
        }
    };

    let mut lua = Lua::new();

    match boot_lua(&mut lua) {
        Ok(_) => (),
        Err(err) => {
            eprintln!("Failed to boot Lua runtime");
            eprintln!("{}", err);
            return to_c_string(format!("Failed to boot Lua runtime, {}", err).to_string());
        }
    };

    let globals = lua.globals();
    let handle_func: LuaFunction = match globals.get("handle") {
        Ok(func) => func,
        Err(err) => {
            eprintln!("Function 'handle' is not defined globally in Lua runtime\n");
            eprintln!("{}", err);
            return to_c_string("".to_string());
        },
    };

    let result: LuaResult<String> = handle_func.call((arg0_str, arg1_str));
    match result {
        Ok(res) => {
            to_c_string(res)
        },
        Err(err) => {
            eprintln!("Failed to call 'handle' function\n");
            eprintln!("{}", err);
            to_c_string("".to_string())
        },
    }
}

#[no_mangle]
pub extern "C" fn main() -> i32 {
    0
}

