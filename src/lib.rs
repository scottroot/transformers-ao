use std::ffi::{c_char, CStr, CString};
use mlua::Lua;
use mlua::prelude::*;
use std::sync::{Mutex, MutexGuard};
use lazy_static::lazy_static;

mod aos;
use aos::aos_process;

mod models;
mod weavedrive;
mod utils;


extern "C" {
    fn ao_log_js(message: *const u8);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn ao_log(message: &str) {
    eprintln!("{}", message);
}

#[cfg(target_arch = "wasm32")]
pub fn ao_log(message: &str) {
    // matching in case the message contains null bytes or something...
    match CString::new(message) {
        Ok(c_message) => {
            let c_message_ptr = c_message.as_ptr() as *const u8;
            unsafe {
                ao_log_js(c_message_ptr);
            }
        }
        Err(e) => {
            eprintln!("Failed to create CString: {}", e);
        }
    }
}


lazy_static! {
    /// A global static ref to a `Mutex` holding an `Option`-wrapped Lua state.
    /// Ensures that subsequent runs on AO will persist any state changes and that
    /// the Lua state is only initialized once.
    static ref LUA_STATE: Mutex<Option<Lua>> = Mutex::new(None);
}

/// Gets the guard to the global Lua state.
///
/// Locks the `LUA_STATE` mutex and returns a guard to it. If the mutex was previously
/// poisoned somehow, it tries to recover by taking the inner value of the poisoned mutex,
/// but if not, it will just freak out.
///
/// # Returns
///
/// A `MutexGuard` to the `Option<Lua>` inside the `LUA_STATE` mutex.
fn get_lua_state() -> MutexGuard<'static, Option<Lua>> {
    match LUA_STATE.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!("Mutex poisoned, trying to recover");
            poisoned.into_inner()
        }
    }
}

/// Converts a Rust `String` to a `CString` and returns it as a raw C string pointer.
///
/// # Arguments
///
/// * `rust_string` - The Rust `String` to be converted.
///
/// # Returns
///
/// A raw pointer to a null-terminated C string (`*const c_char`).
///
/// # Panics
///
/// This function will panic if your `rust_string` contains any null bytes.
pub fn to_c_string(rust_string: String) -> *const c_char {
    CString::new(rust_string).unwrap().into_raw()
}

/// Initialize global Lua state once.
///
/// This function initializes the global Lua state if it has not been initialized yet.
///
/// # Returns
///
/// * () if successful
/// * `LuaError` if failure
fn boot_lua() -> LuaResult<()> {
    let mut lua_lock = get_lua_state();
    if lua_lock.is_none() {
        *lua_lock = Some(Lua::new());
    } else {
        return Ok(())
    };
    let lua = lua_lock.as_ref().expect("Lua state is not initialized");
    let globals = lua.globals();

    globals.set("println", lua.create_function(|_, s: String| {
        println!("{}", s);
        Ok(())
    })?)?;

    weavedrive::preload(lua)?;
    models::bert::preload(lua)?;
    utils::preload_serde_json(lua)?;
    utils::mock_non_deterministic_globals(lua)?;
    aos_process::preload(&lua)?;

    lua.load(r#"Handlers.add("pingpong", Handlers.utils.hasMatchingTag("Action", "ping"), Handlers.utils.reply("pong"))"#).exec()?;

    Ok(())
}

#[no_mangle]
pub extern "C" fn handle(arg0: *const c_char, arg1: *const c_char) -> *const c_char {
    let arg0_str = unsafe {
        if arg0.is_null() {
            ao_log("Handle arg0 is null");
            return to_c_string("".to_string());
        }
        match CStr::from_ptr(arg0).to_str() {
            Ok(s) => s,
            Err(err) => {
                ao_log(&format!("Handle arg0 is invalid UTF-8 | {}", err));
                return to_c_string("".to_string())
            },
        }
    };
    let arg1_str = unsafe {
        if arg1.is_null() {
            ao_log("Handle arg1 is null");
            return to_c_string("".to_string());
        }
        match CStr::from_ptr(arg1).to_str() {
            Ok(s) => s,
            Err(err) => {
                ao_log(&format!("Handle arg1 is invalid UTF-8 | {}", err));
                return to_c_string("".to_string())
            },
        }
    };
    match boot_lua() {
        Ok(_) => (),
        Err(err) => {
            ao_log(&format!("Failed to boot Lua runtime | {}", err));
            return to_c_string(format!("Failed to boot Lua runtime, {}", err).to_string());
        }
    };
    let lua_lock = get_lua_state();
    let lua = lua_lock.as_ref().expect("Lua state is not initialized");

    let globals = lua.globals();
    let handle_func: LuaFunction = match globals.get(".loader") {
        Ok(func) => func,
        Err(err) => {
            ao_log(&format!("Function 'handle' is not defined globally in Lua runtime | {}", err));
            return to_c_string("".to_string());
        },
    };

    let result: LuaResult<String> = handle_func.call((arg0_str, arg1_str));
    match result {
        Ok(res) => to_c_string(res),
        Err(err) => {
            ao_log(&format!("Failed to call 'handle' function | {}", err));
            to_c_string("".to_string())
        },
    }
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;

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
    fn test_boot_lua() {
        let result = boot_lua();
        assert!(result.is_ok());

        let mut lua_lock = get_lua_state();
        let lua = lua_lock.as_ref().expect("Lua state is not initialized");

        // Test "println" function
        let println: mlua::Function = lua.globals().get("println").unwrap();
        let result = println.call::<_, ()>("'test println message'".to_string());
        assert!(result.is_ok(), "test_boot_lua - println");

        // Test packages loaded
        let package: LuaTable = lua.globals().get("package").unwrap();
        let loaded: LuaTable = package.get("loaded").unwrap();
        assert!(loaded.contains_key(".pretty").unwrap());
        assert!(loaded.contains_key(".base64").unwrap());
        assert!(loaded.contains_key(".chance").unwrap());
        assert!(loaded.contains_key(".dump").unwrap());
        assert!(loaded.contains_key(".utils").unwrap());
        assert!(loaded.contains_key(".handlers-utils").unwrap());
        assert!(loaded.contains_key(".handlers").unwrap());
        assert!(loaded.contains_key(".stringify").unwrap());
        assert!(loaded.contains_key(".eval").unwrap());
        assert!(loaded.contains_key(".default").unwrap());
        assert!(loaded.contains_key(".handlers").unwrap());
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
    //                 Handlers.utils.hasMatchingTag('Action', 'marco'),
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
    //
    //     let msg = serde_json::to_string(&serde_json::json!({
    //         "Target": "AOS",
    //         "Owner": "FOOBAR",
    //         "Block-Height": "1000",
    //         "Id": "1234xyxfoo",
    //         "Module": "WOOPAWOOPA",
    //         "Tags": [{"name": "Action", "value": "marco"}],
    //     })).unwrap();
    //     globals.set("msg", msg).unwrap();
    //
    //     let handle_2: LuaResult<LuaValue> = lua.load("return handle(msg, env)").eval();
    //     assert!(handle_2.is_ok());
    //
    //     let result = handle_2.unwrap().to_string().unwrap();
    //     let ujson: serde_json::Value = serde_json::from_str(&result).unwrap();
    //     let result = ujson.get("response")
    //         .and_then(|r| r.get("Output"))
    //         .and_then(|m| m.get("data"))
    //         .unwrap();
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