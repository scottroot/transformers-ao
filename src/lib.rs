#![allow(dead_code)]
#![allow(unused_imports)]
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use mlua::Lua;
use mlua::prelude::*;

mod aos;
use aos::preloader;
mod models;
mod weavedrive;

// #[cfg(feature = "logging")]
#[macro_export]
macro_rules! ao_log {
    ($($arg:tt)*) => {
        eprintln!($($arg)*);
    };
}

// #[cfg(not(feature = "logging"))]
// #[macro_export]
// macro_rules! ao_log {
//     ($($arg:tt)*) => {};
// }


fn create_io_table(lua: &Lua) -> LuaResult<LuaTable> {
    let io_table = lua.create_table()?;

    let open = lua.create_function(|_, (filename, mode): (String, String)| {
        let fd = weavedrive::open(&filename, &mode);
        if fd == 0 {
            return Ok(None);
        }
        Ok(Some(fd))
    })?;
    io_table.set("open", open)?;

    let read = lua.create_function(|_, fd: i32| {
        let chunk_size = 1024;
        let mut buffer = Vec::new();
        let mut total_bytes_read = 0;

        loop {
            let mut temp_buffer = vec![0u8; chunk_size];
            let bytes_read = weavedrive::read(fd, &mut temp_buffer);
            if bytes_read < 0 {
                return Ok(None);
            }
            if bytes_read == 0 {
                break;
            }
            buffer.extend_from_slice(&temp_buffer[..bytes_read as usize]);
            total_bytes_read += bytes_read as usize;

            if bytes_read < chunk_size as i32 {
                break;
            }
        }
        // Get rid of extra bytes from cases when chunk size not matched to file length
        buffer.truncate(total_bytes_read);
        Ok(Some(String::from_utf8_lossy(&buffer).to_string()))
    })?;
    io_table.set("read", read)?;

    let close = lua.create_function(|_, fd: i32| {
        weavedrive::close(fd);
        Ok(())
    })?;
    io_table.set("close", close)?;

    Ok(io_table)
}

fn to_c_string(rust_string: String) -> *const c_char {
    CString::new(rust_string).unwrap().into_raw()
}

fn boot_lua(lua: &mut Lua) -> LuaResult<()> {
    let globals = lua.globals();
    let _print: LuaFunction = lua.create_function(|_, s: String| {
        ao_log!("{}", s);
        Ok(())
    })?;
    globals.set("println", _print)?;

    let _stringify: LuaFunction = lua.create_function(|_, t: LuaTable| {
        let json_str = serde_json::to_string(&t).map_err(LuaError::external)?;
        Ok(json_str)
    })?;
    globals.set("_stringify", _stringify)?;

    let io_table = create_io_table(&lua)?;
    globals.set("io", io_table)?;

    models::bert::preload(lua)?;

    globals.set("lua_bundle", lua.create_table()?)?;
    preloader::exec(lua, "main", include_str!("aos/main.lua"))?;

    preloader::set_loaded(lua, "ao", include_str!("aos/ao.lua"))?;
    preloader::set_loaded(lua, "json", include_str!("aos/json.lua"))?;
    preloader::set_loaded(lua, "weavedrive", include_str!("aos/weavedrive.lua"))?;

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
            ao_log!("Handle arg0 is null");
            return to_c_string("".to_string());
        }
        match CStr::from_ptr(arg0).to_str() {
            Ok(s) => s,
            Err(err) => {
                ao_log!("Handle arg0 is invalid UTF-8\\n");
                ao_log!("{}", err);
                return to_c_string("".to_string())
            },
        }
    };
    let arg1_str = unsafe {
        if arg1.is_null() {
            ao_log!("Handle arg1 is null");
            return to_c_string("".to_string());
        }
        match CStr::from_ptr(arg1).to_str() {
            Ok(s) => s,
            Err(err) => {
                ao_log!("Handle arg1 is invalid UTF-8\\n");
                ao_log!("{}", err);
                return to_c_string("".to_string())
            },
        }
    };

    let mut lua = Lua::new();

    match boot_lua(&mut lua) {
        Ok(_) => (),
        Err(err) => {
            ao_log!("Failed to boot Lua runtime");
            ao_log!("{}", err);
            return to_c_string(format!("Failed to boot Lua runtime, {}", err).to_string());
        }
    };

    let globals = lua.globals();
    let handle_func: LuaFunction = match globals.get("handle") {
        Ok(func) => func,
        Err(err) => {
            ao_log!("Function 'handle' is not defined globally in Lua runtime\n");
            ao_log!("{}", err);
            return to_c_string("".to_string());
        },
    };

    let result: LuaResult<String> = handle_func.call((arg0_str, arg1_str));
    match result {
        Ok(res) => {
            to_c_string(res)
        },
        Err(err) => {
            ao_log!("Failed to call 'handle' function\n");
            ao_log!("{}", err);
            to_c_string("".to_string())
        },
    }
}

#[no_mangle]
pub extern "C" fn main() -> i32 {
    0
}

