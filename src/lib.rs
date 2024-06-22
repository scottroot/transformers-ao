#![allow(dead_code)]
#![allow(unused_imports)]
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use mlua::Lua;
use mlua::prelude::*;
use std::sync::{Mutex, MutexGuard};
use lazy_static::lazy_static;

mod aos;
use aos::preloader;
use crate::aos::preloader::LoadType;

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

lazy_static! {
    static ref LUA_STATE: Mutex<Option<Lua>> = Mutex::new(None);
}

fn get_lua_state() -> MutexGuard<'static, Option<Lua>> {
    match LUA_STATE.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Mutex poisoned, trying to recover");
            poisoned.into_inner()
        }
    }
}
pub fn to_c_string(rust_string: String) -> *const c_char {
    CString::new(rust_string).unwrap().into_raw()
}

// fn boot_lua(lua: &mut Lua) -> LuaResult<()> {
fn boot_lua() -> LuaResult<()> {
    // let mut lua_lock = LUA_STATE.lock().expect("Failed to lock GLOBAL_LUA");
    let mut lua_lock = get_lua_state();
    if lua_lock.is_none() {
        *lua_lock = Some(Lua::new());
    } else {
        return Ok(())
    };
    let lua = lua_lock.as_ref().expect("Lua state is not initialized");

    let globals = lua.globals();
    globals.set("println", lua.create_function(|_, s: String| {
        ao_log!("{}", s);
        Ok(())
    })?)?;

    globals.set("_stringify", lua.create_function(|_, t: LuaTable| {
        let json_str = serde_json::to_string(&t).map_err(LuaError::external)?;
        Ok(json_str)
    })?)?;

    weavedrive::preload(lua)?;

    models::bert::preload(lua)?;

    preloader::exec(lua, "main", include_str!("aos/main.lua"))?;

    preloader::set_loaded(lua, "ao", include_str!("aos/ao.lua"), LoadType::Table)?;
    preloader::set_loaded(lua, "json", include_str!("aos/json.lua"), LoadType::Table)?;

    // Not including Lua version of WeaveDrive anymore since it is handled by weavedrive.rs
    // preloader::set_loaded(lua, "weavedrive", include_str!("aos/weavedrive.lua"))?;

    // Unit Tests for AOS Lua code is in aos/preloader.rs
    // globals.set("lua_bundle", lua.create_table()?)?;
    // preloader::set_bundle(lua, ".pretty", include_str!("aos/pretty.lua"))?;
    // preloader::set_bundle(lua, ".base64", include_str!("aos/base64.lua"))?;
    // preloader::set_bundle(lua, ".chance", include_str!("aos/chance.lua"))?;
    // preloader::set_bundle(lua, ".dump", include_str!("aos/dump.lua"))?;
    // preloader::set_bundle(lua, ".utils", include_str!("aos/utils.lua"))?;
    // preloader::set_bundle(lua, ".handlers-utils", include_str!("aos/handlers-utils.lua"))?;
    // preloader::set_bundle(lua, ".handlers", include_str!("aos/handlers.lua"))?;
    // preloader::set_bundle(lua, ".stringify", include_str!("aos/stringify.lua"))?;
    // preloader::set_bundle(lua, ".eval", include_str!("aos/eval.lua"))?;
    // preloader::set_bundle(lua, ".default", include_str!("aos/default.lua"))?;
    // preloader::set_bundle(lua, ".handlers", include_str!("aos/handlers.lua"))?;

    preloader::set_loaded(&lua, ".pretty", include_str!("aos/pretty.lua"), LoadType::Table)?;
    preloader::set_loaded(&lua, ".base64", include_str!("aos/base64.lua"), LoadType::Table)?;
    preloader::set_loaded(&lua, ".chance", include_str!("aos/chance.lua"), LoadType::Table)?;
    preloader::set_loaded(&lua, ".dump", include_str!("aos/dump.lua"), LoadType::Function)?;
    preloader::set_loaded(&lua, ".utils", include_str!("aos/utils.lua"), LoadType::Table)?;
    preloader::set_loaded(&lua, ".handlers-utils", include_str!("aos/handlers-utils.lua"), LoadType::Table)?;
    preloader::set_loaded(&lua, ".handlers", include_str!("aos/handlers.lua"), LoadType::Table)?;
    preloader::set_loaded(&lua, ".stringify", include_str!("aos/stringify.lua"), LoadType::Table)?;
    preloader::set_loaded(&lua, ".eval", include_str!("aos/eval.lua"), LoadType::Function)?;
    preloader::set_eval_lua(&lua)?;
    preloader::set_loaded(&lua, ".default", include_str!("aos/default.lua"), LoadType::Function)?;
    preloader::set_loaded(&lua, ".handlers", include_str!("aos/handlers.lua"), LoadType::Table)?;

    preloader::set_loaded(lua, ".process", include_str!("aos/process.lua"), LoadType::Table)?;
    // preloader::exec(lua, "loader", include_str!("aos/loader.lua"))?;
    // preloader::set_loaded(lua, ".loader", include_str!("aos/loader.lua"), LoadType::Function)?;
    let loader: LuaFunction = lua.load(include_str!("aos/loader.lua")).eval()?;
    globals.set(".loader", loader)?;

    lua.load(r#"Handlers.add("pingpong", Handlers.utils.hasMatchingTag("Action", "ping"), Handlers.utils.reply("pong"))"#).exec()?;

    Ok(())
    // return true
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

    // let mut lua = Lua::new();
    // match boot_lua(&mut lua) {
    match boot_lua() {
        Ok(_) => (),
        Err(err) => {
            ao_log!("Failed to boot Lua runtime");
            ao_log!("{}", err);
            return to_c_string(format!("Failed to boot Lua runtime, {}", err).to_string());
        }
    };
    let lua_lock = get_lua_state();
    let lua = lua_lock.as_ref().expect("Lua state is not initialized");

    let globals = lua.globals();
    let handle_func: LuaFunction = match globals.get(".loader") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;
    use mlua::chunk;

    #[test]
    fn test_to_c_string() {
        let rust_string = String::from("hello");
        let c_string_ptr = to_c_string(rust_string);

        unsafe {
            let c_str = CStr::from_ptr(c_string_ptr);
            assert_eq!(c_str.to_str().unwrap(), "hello");
            let _ = CString::from_raw(c_string_ptr as *mut c_char);
        }
    }

    #[test]
    fn test_main() {
        let result = main();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_boot_lua() {

        // let mut lua = Lua::new();
        // let result = boot_lua(&mut lua);
        let result = boot_lua();
        assert!(result.is_ok());

        let mut lua_lock = get_lua_state();
        let lua = lua_lock.as_ref().expect("Lua state is not initialized");

        // Test "println" function
        let println: mlua::Function = lua.globals().get("println").unwrap();
        let result = println.call::<_, ()>("'test println message'".to_string());
        assert!(
            result.is_ok(),
            "test_boot_lua - println"
        );

        // Test "_stringify" function
        let stringify: mlua::Function = lua.globals().get("_stringify").unwrap();
        let table = lua.create_table().unwrap();
        table.set("Name", "Alice").unwrap();
        let result: String = stringify.call(table).unwrap();
        assert_eq!(result, r#"{"Name":"Alice"}"#);

        // Test if "lua_bundle" table is available
        let lua_bundle: LuaTable = lua.globals().get("lua_bundle").unwrap();
        assert!(lua_bundle.contains_key(".pretty").unwrap());
        assert!(lua_bundle.contains_key(".base64").unwrap());
        assert!(lua_bundle.contains_key(".chance").unwrap());
        assert!(lua_bundle.contains_key(".dump").unwrap());
        assert!(lua_bundle.contains_key(".utils").unwrap());
        assert!(lua_bundle.contains_key(".handlers-utils").unwrap());
        assert!(lua_bundle.contains_key(".handlers").unwrap());
        assert!(lua_bundle.contains_key(".stringify").unwrap());
        assert!(lua_bundle.contains_key(".eval").unwrap());
        assert!(lua_bundle.contains_key(".default").unwrap());
        assert!(lua_bundle.contains_key(".handlers").unwrap());

        // Test if "process" module is loaded
        // let process: LuaFunction = lua.load("return require('.process')").eval().unwrap();
        // assert!(process.type_name() == "function");

        // // Test if the final script execution was successful
        // let handlers_add: LuaFunction = lua.load(r#"return Handlers.add"#).eval().unwrap();
        // assert!(handlers_add.type_name() == "function");
    }

    // #[test]
    // fn test_lua_message_handling() {
    //     let mut lua = Lua::new();
    //     let result = boot_lua(&mut lua);
    //     assert!(result.is_ok());
    //
    //     let globals = lua.globals();
    //
    //     let msg = serde_json::to_string(&serde_json::json!({
    //         "Target": "AOS",
    //         "Owner": "FOOBAR",
    //         "Block-Height": "1000",
    //         "Id": "1234xyxfoo",
    //         "Module": "WOOPAWOOPA",
    //         "Tags": [{"name": "Action", "value": "Eval"}],
    //         "Data": r#"
    //             Handlers.add('marcopolo',
    //                 Handlers.utils.hasMatchingData('marco'),
    //                 function (Msg)
    //                     return('polo')
    //                 end
    //             )
    //         "#
    //     })).unwrap();
    //     globals.set("msg", msg).unwrap();
    //
    //     let env = serde_json::to_string(&serde_json::json!({
    //         "Process": {
    //             "Id": "AOS",
    //             "Owner": "FOOBAR",
    //             "Tags": [{"name": "Name", "value": "Thomas"}]
    //         }
    //     })).unwrap();
    //     globals.set("env", env).unwrap();
    //     let handle_1: LuaResult<String> = lua.load("handle(msg, env)").eval();
    //     assert!(handle_1.is_ok());
    //     // match handle_1 {
    //     //     Ok(x) => println!("MSG 1:\n{}", x),
    //     //     Err(e) => eprintln!("{}", e)
    //     // };
    //
    //
    //
    //
    //
    //     let msg = serde_json::to_string(&serde_json::json!({
    //         "Target": "AOS",
    //         "Owner": "FOOBAR",
    //         "Block-Height": "1000",
    //         "Id": "1234xyxfoo",
    //         "Module": "WOOPAWOOPA",
    //         // "Tags": [{"name": "Action", "value": "Eval"}],
    //         "Data": "marco"
    //     })).unwrap();
    //     globals.set("msg", msg).unwrap();
    //
    //     let handle_2: LuaResult<LuaValue> = lua.load("return handle(msg, env)").eval();
    //     assert!(handle_2.is_ok());
    //
    //     let result = handle_2.unwrap().to_string().unwrap();
    //     let ujson: serde_json::Value = serde_json::from_str(&result).unwrap();
    //     println!("********\n>>>>{}<<<<\n********", ujson);
    //     let result = ujson.get("response")
    //         .and_then(|r| r.get("Output"))
    //         .and_then(|m| m.get("data"))
    //         .unwrap();
    //     // ujson["Messages"].as_array().unwrap().last().unwrap()["Data"].as_str().unwrap().to_string()
    //     println!(">>>>{}<<<<", result);
    //     assert_eq!(result, "polo");
    //
    //
    // }

    // const ENV: &str = include_str!("test_env.json");

    // #[test]
    // fn test_handle() {
    //     let env_content: &str = include_str!("test_env.json");
    //     let env: *const c_char = to_c_string(env_content.to_string());
    //     let msg_content: &str = r#"{
    //       "Id": "FOO",
    //       "Owner": "tom",
    //       "Target": "AOS",
    //       "Tags": [
    //         { "name": "Action", "value": "Eval" }
    //       ],
    //       "Module": "1234",
    //       "Block-Height": "1000"
    //       "Data": "return('Test response');"
    //     }"#;
    //     let msg: *const c_char = to_c_string(msg_content.to_string());
    //     let outcome = handle(msg, env);
    //     let result = unsafe {
    //         match CStr::from_ptr(outcome).to_str() {
    //             Ok(s) => s,
    //             Err(err) => {
    //                 eprintln!("Handle result is invalid UTF-8\\n");
    //                 eprintln!("{}", err);
    //                 return
    //             },
    //         }
    //     };
    //     println!("{}", result);
    //     return
    //     // assert_eq!(expected_outcome, outcome); // expected_outcome is the expected result of the function with the provided arguments
    // }
}