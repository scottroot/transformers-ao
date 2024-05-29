#![no_main]
use libc::{printf, c_char};

mod aos;
use aos::lua_program_handle;
use aos::lua_src_ao;
use aos::lua_src_base64;
use aos::lua_src_chance;
use aos::lua_src_dump;
use aos::lua_src_default;
use aos::lua_src_eval;
use aos::lua_src_handlers;
use aos::lua_src_handlers_utils;
use aos::lua_src_json;
use aos::lua_src_pretty;
use aos::lua_src_process;
use aos::lua_src_stringify;
use aos::lua_src_utils;

#[export_name = "handle"]
pub extern "C" fn handle(msg_json: *const c_char, ao_json: *const c_char) -> *const c_char {
    let msg_str = unsafe {
        assert!(!msg_json.is_null());
        CStr::from_ptr(msg_json).to_str().unwrap()
    };

    let ao_str = unsafe {
        assert!(!ao_json.is_null());
        CStr::from_ptr(ao_json).to_str().unwrap()
    };

    match runtime(msg_str, ao_str) {
        Ok(r) => {
            let msg = r.to_string() + "\0";
            msg.as_ptr() as *const c_char
        }
        Err(e) => {
            let msg = e.to_string() + "\0";
            msg.as_ptr() as *const c_char
        }
    }
}
