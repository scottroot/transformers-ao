use std::env;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions, read_to_string};
use std::io::prelude::*;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use walkdir::WalkDir;

fn encode_hex_literals(source: &str) -> String {
    source
        .as_bytes()
        .iter()
        .map(|b| format!("0x{:02x}", b))
        .collect::<Vec<String>>()
        .join(", ")
}

struct LuaFile {
    filepath: String,
    basepath: String,
    module_name: String,
}

impl LuaFile {
    fn new(filepath: &str, basepath: Option<String>) -> LuaFile {
        let basepath = basepath.unwrap_or_else(|| 
          Path::new(filepath).file_name().unwrap().to_str().unwrap().to_string())
        );
        let mut module_name = Path::new(&filepath)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .replace(&*basepath, "")
            .replace("/", ".");
        if module_name.starts_with(".src") {
            module_name = module_name.replacen(".src", "", 1);
        }
        LuaFile {
            filepath: filepath.to_string(),
            basepath,
            module_name,
        }
    }
}

pub fn make_function_declarations() -> String {
    let functions: Vec<(&str, (&str, Vec<&str>))> = vec![(
        "handle",
        ("string", vec![
            "string",
            "string"
        ])
    )];
    let mut wasm_functions = Vec::new();
    for (name, (mut return_type, args)) in functions {
        let mut arguments = Vec::new();
        let mut push_arguments = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            match *arg {
                "int" => {
                    arguments.push(format!("int arg_{}", i));
                    push_arguments.push(format!("  lua_pushnumber(wasm_lua_state, arg_{});", i));
                },
                "string" => {
                    arguments.push(format!("const char* arg_{}", i));
                    push_arguments.push(format!("  lua_pushstring(wasm_lua_state, arg_{});", i));
                },
                _ => {}
            }
        }

        let mut failed_return_value=  "";
        let mut capture_return_value = "";
        if return_type == "int" {
            failed_return_value = "return 0;";
            capture_return_value = r#"  if (lua_isinteger(wasm_lua_state, -1)) {
    int return_value = lua_tointeger(wasm_lua_state, -1);
    lua_settop(wasm_lua_state, 0);
    return return_value;
  }
  return 0;"#;
        } else if return_type == "string" {
            return_type = "const char* ";
            failed_return_value = r#"return "";"#;
            capture_return_value = r#"  if (lua_isstring(wasm_lua_state, -1)) {
    const char* return_value = lua_tostring(wasm_lua_state, -1);
    lua_settop(wasm_lua_state, 0);
    return return_value;
  }
  return "";"#;
        }

        let function = format!(r#"
EMSCRIPTEN_KEEPALIVE
{} {}({}) {{
  if (wasm_lua_state == NULL) {{
    wasm_lua_state = luaL_newstate();
    boot_lua(wasm_lua_state);
  }}
  // Push arguments
  lua_getglobal(wasm_lua_state, "{}");
  if (!lua_isfunction(wasm_lua_state, -1)) {{
    printf("function {} is not defined globaly in lua runtime\\n");
    lua_settop(wasm_lua_state, 0);
    {}
  }}
{}

  // Call lua function
  if (lua_pcall(wasm_lua_state, {}, 1, 0)) {{
    printf("failed to call {} function\\n");
    printf("error: %s\\n", lua_tostring(wasm_lua_state, -1));
    lua_settop(wasm_lua_state, 0);
    {}
  }}
  // Handle return values
{}
}}
        "#,
            return_type, name, arguments.join(", "),
            name,                       // 4
            name,                       // 5
            failed_return_value,        // 6
            push_arguments.join("\n"),  // 7
            push_arguments.len(),       // 8
            name,                       // 9
            failed_return_value,        // 10
            capture_return_value        // 11
        );
        println!("{}", function);
        wasm_functions.push(function);
    }

    return wasm_functions.join("\n")
}
pub fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_arch != "wasm32" || target_os != "emscripten" {
        return
    }

    println!("Running build script for wasm32-unknown-emscripten...");

    let local_lua_files_path: String = String::from("/src/container/process");
    let mut lua_files: Vec<LuaFile> = Vec::new();
    let bundle_files = WalkDir::new(&*local_lua_files_path).min_depth(0).max_depth(5);

    for item in bundle_files {
        let item = item.unwrap();
        if item.file_type().is_file() {
            let display_path = item.path().display().to_string();
            match item.path().extension().and_then(OsStr::to_str) {
                Some("lua") => lua_files.push(LuaFile::new(&*display_path, Some(local_lua_files_path.clone()))),
                // Some("so") => library_files.push(LuaFile::new(&*display_path, &*basepath)),
                _ => (),
            }
        }
    }

    let mut lua_file_injections: Vec<String> = Vec::new();
    for (i, f) in lua_files.iter().enumerate() {
        let mut file = BufReader::new(File::open(&f.filepath).unwrap());
        let mut lines: Vec<String> = Vec::new();
        file.lines().for_each(|line| lines.push(line.unwrap()));

        if lines[0].starts_with("\u{feff}") { // Checking for BOM
            lines[0] = lines[0][3..].to_string();
        } else if lines[0].starts_with('#') { // Checking for shebang
            lines.remove(0);
        }

        let file_string = format!(
            r#"static const unsigned char lua_require_{}[] = {{{}}};
  lua_pushlstring(L, (const char*)lua_require_{}, sizeof(lua_require_{}));
  lua_setfield(L, -2, "{}");"#,
            i, encode_hex_literals(&lines.join("\n")), i, i, f.module_name
        );
        lua_file_injections.push(file_string);
    }

    let lua_base_program = read_to_string("/opt/main.lua").unwrap();
    let lua_entry_program = read_to_string("/opt/loader.lua").unwrap();

    let mut c_program_file = File::open("build/template.c").unwrap();
    let mut c_program = String::new();
    c_program_file.read_to_string(&mut c_program).unwrap();

    c_program = c_program.replace("__LUA_BASE__", &encode_hex_literals(&lua_base_program));
    c_program = c_program.replace("__LUA_MAIN__", &encode_hex_literals(&lua_entry_program));

    c_program = c_program.replace(
        "__LUA_FUNCTION_DECLARATIONS__",
        &make_function_declarations()
    );
    c_program = c_program.replace(
        "__INJECT_LUA_FILES__",
        &lua_file_injections.join("\n")
    );

    let mut compile_c_file = File::create("/opt/compile.c")
        .expect("Failed to create file");
    write!(&mut compile_c_file, "{}", c_program)
        .expect("Failed to write to file");
}
