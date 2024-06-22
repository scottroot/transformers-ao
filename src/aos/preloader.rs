use mlua::prelude::*;

pub enum LoadType {
    Function,
    Table,
}

/// Sets a Lua value in the `loaded` table within the `package` table in the Lua global scope.
///
/// # Arguments
///
/// * `lua` - A reference to the Lua state.
/// * `name` - The name to assign to the value in the `loaded` table.
/// * `content` - The Lua code to load and evaluate.
/// * `load_type` - The type of value to be loaded, either `LoadType::Function` or `LoadType::Table`.
///
/// # Errors
///
/// Returns a `LuaResult<()>` which will contain an error if the loading or setting of the value fails.
pub fn set_loaded(lua: &Lua, name: &str, content: &str, load_type: LoadType) -> LuaResult<()> {
    let package: LuaTable = lua.globals().get("package")?;
    let loaded: LuaTable = package.get("loaded")?;
    match load_type {
        LoadType::Function => {
            let value: LuaFunction = lua.load(content).set_name(name).eval()?;
            loaded.set(name, value)?;
        },
        LoadType::Table => {
            let value: LuaTable = lua.load(content).set_name(name).eval()?;
            loaded.set(name, value)?;
        }
    }
    Ok(())
}
pub fn set_bundle(lua: &Lua, name: &str, content: &str) -> LuaResult<()> {
    let bundle: LuaTable = lua.globals().get("lua_bundle")?;
    bundle.set(name, content)?;
    Ok(())
}

pub fn exec(lua: &Lua, name: &str, content: &str) -> LuaResult<()> {
    lua.load(content).set_name(name).exec()?;
    Ok(())
}

pub fn set_eval_lua(lua: &Lua) -> LuaResult<()> {
        let lua_bundle: LuaTable = match lua.globals().get::<_, LuaTable>("lua_bundle") {
            Ok(x) => x,
            Err(_) => {
                lua.globals().set("lua_bundle", lua.create_table()?)?;
                lua.globals().get("lua_bundle")?
            }
        };
        lua_bundle.set(".eval-cb", lua.create_function(|lua_cb, msg: LuaTable| {
            let globals: LuaTable = lua_cb.globals();
            let package: LuaTable = globals.get("package")?;
            let loaded: LuaTable = package.get("loaded")?;

            let stringify_table: LuaTable = match loaded.get::<_, LuaTable>(".stringify") {
                Ok(x) => x,
                Err(e) => {
                    println!("Error getting .stringify during .eval-cb");
                    println!("{}", &e.to_string());
                    return Err(e)
                }
            };
            let stringify: LuaFunction = stringify_table.get("format")?;

            let ao: LuaTable = match globals.get::<_, LuaTable>("_AO") {
                Ok(x) => x,
                Err(e) => {
                    println!("Error getting _AO during .eval-cb");
                    println!("{}", &e.to_string());
                    return Err(e)
                }
            };
            let outbox: LuaTable = match ao.get::<_, LuaTable>("outbox") {
                Ok(x) => x,
                Err(e) => {
                    println!("Error getting outbox during eval, {}", &e.to_string());
                    return Err(e)
                }
            };
            let expr: String = match msg.get::<_, String>("Data") {
                Ok(s) => {
                    if s == "" {
                        outbox.set("Error", "No Data to Eval, received empty string.")?;
                        println!("No Data to Eval, received empty string.");
                        return Ok(());
                    }
                    s
                },
                Err(e) => {
                    outbox.set("Error", format!("Error evaluating Data. {}", &e.to_string()))?;
                    println!("Error evaluating Data. {}", &e.to_string());
                    return Ok(());
                }
            };

            // let result: LuaResult<LuaFunction> = lua_cb.load(&format!("return {}", expr))
            //     .set_name("aos")
            //     .eval();
            // let result: LuaResult<LuaValue> = lua_cb.load(expr).set_name("aos").eval();

            // let func = match result {
            //     Ok(f) => f,
            //     Err(_) => match lua_cb.load(&expr).set_name("aos").eval() {
            //         Ok(f) => f,
            //         Err(e) => {
            //             let outbox: LuaTable = match ao.get::<_, LuaTable>("outbox") {
            //                 Ok(x) => x,
            //                 Err(e) => {
            //                     println!("Error getting outbox during .eval-cb, {}", &e.to_string());
            //                     return Err(e)
            //                 }
            //             };
            //             outbox.set("Error", e.to_string())?;
            //             return Ok(());
            //         }
            //     },
            // };

            // match func.call::<_, LuaValue>(()) {
            //     Ok(output) => {
            //         // let outbox: LuaTable = ao.get("outbox")?;
            //         // let outbox: LuaTable = match ao.get::<_, LuaTable>("outbox") {
            //         //     Ok(x) => x,
            //         //     Err(e) => {
            //         //         println!("Error getting outbox during .eval-cb func call, {}", &e.to_string());
            //         //         return Err(e)
            //         //     }
            //         // };
            //         let data_table = lua_cb.create_table()?;
            //
            //         let data_table_json = if let LuaValue::Table(ref val) = output {
            //             serde_json::to_string(val).unwrap_or_else(|e| {
            //                 eprintln!("Serialization error: {:?}", e);
            //                 "undefined".to_string()
            //             })
            //         } else {
            //             "undefined".to_string()
            //         };
            //
            //         let data_table_output = if let LuaValue::Table(_) = output {
            //             stringify.call::<_, String>(output.clone())?
            //         } else {
            //             match output {
            //                 LuaValue::String(ref s) => s.to_str()?.to_string(),
            //                 LuaValue::Number(n) => n.to_string(),
            //                 LuaValue::Integer(i) => i.to_string(),
            //                 LuaValue::Boolean(b) => b.to_string(),
            //                 _ => match output.to_string() {
            //                     Ok(s) => s,
            //                     Err(e) => {
            //                         outbox.set("Error", e.to_string())?;
            //                         return Ok(());
            //                     }
            //                 }
            //             }
            //         };
            //
            //         let prompt: String = lua_cb.load("return Prompt()").eval().unwrap_or_else(|_| "aos> ".to_string());
            //
            //         data_table.set("json", data_table_json)?;
            //         data_table.set("output", data_table_output)?;
            //         data_table.set("prompt", prompt)?;
            //
            //         outbox.set("Output", data_table)?;
            //     },
            //     Err(e) => {
            //         let outbox: LuaTable = ao.get("outbox")?;
            //         outbox.set("Error", e.to_string())?;
            //     },
            // }

            let result: LuaResult<LuaValue> = lua_cb.load(expr).set_name("aos").eval();
            match result {
                Ok(output) => {
                    let data_table = lua_cb.create_table()?;

                    let data_table_json = if let LuaValue::Table(ref val) = output {
                        serde_json::to_string(val).unwrap_or_else(|e| {
                            eprintln!("Serialization error: {:?}", e);
                            "undefined".to_string()
                        })
                    } else {
                        "undefined".to_string()
                    };

                    let data_table_output = if let LuaValue::Table(_) = output {
                        stringify.call::<_, String>(output.clone())?
                    } else {
                        match output {
                            LuaValue::String(ref s) => s.to_str()?.to_string(),
                            LuaValue::Number(n) => n.to_string(),
                            LuaValue::Integer(i) => i.to_string(),
                            LuaValue::Boolean(b) => b.to_string(),
                            _ => match output.to_string() {
                                Ok(s) => s,
                                Err(e) => {
                                    outbox.set("Error", e.to_string())?;
                                    return Ok(());
                                }
                            }
                        }
                    };

                    let prompt: String = lua_cb.load("return Prompt()").eval().unwrap_or_else(|_| "aos> ".to_string());

                    data_table.set("json", data_table_json)?;
                    data_table.set("output", data_table_output)?;
                    data_table.set("prompt", prompt)?;

                    outbox.set("Output", data_table)?;
                },
                Err(e) => {
                    let outbox: LuaTable = ao.get("outbox")?;
                    outbox.set("Error", e.to_string())?;
                },
            }

            Ok(())
        })?)?;

        let loaded: LuaTable = lua.globals().get::<_, LuaTable>("package")?.get("loaded")?;
        loaded.set(".eval", lua.create_function(move |lua_cb, ao_instance: LuaTable| {
            lua_cb.globals().set("_AO", ao_instance)?;
            let eval_cb: LuaFunction = lua_cb.globals().get::<_, LuaTable>("lua_bundle")?.get(".eval-cb")?;
            Ok(eval_cb)
        })?)?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::fmt::Error;
    use super::*;
    use mlua::Lua;
    use crate::ao_log;

    #[test]
    fn test_set_loaded() {
        let lua = Lua::new();
        let content = r#"
            return {
                hello = function()
                    return "world"
                end
            }
        "#;

        let result = set_loaded(&lua, "hello_module", content, LoadType::Table);
        assert!(result.is_ok());

        let hello: mlua::Function = lua
            .load("return require('hello_module').hello")
            .eval()
            .unwrap();
        let result: String = hello.call(()).unwrap();
        assert_eq!(result, "world");
    }

    #[test]
    fn test_set_bundle() {
        let lua = Lua::new();
        lua.globals().set("lua_bundle", lua.create_table().unwrap()).unwrap();

        let content = "some_test_bundle_content";

        let result = set_bundle(&lua, "test_bundle", content);
        assert!(result.is_ok());

        let lua_bundle: LuaTable = lua.globals().get("lua_bundle").unwrap();
        let stored_content: String = lua_bundle.get("test_bundle").unwrap();
        assert_eq!(stored_content, content);
    }

    #[test]
    fn test_exec() {
        let lua = Lua::new();

        let content = r#"
            function hello()
                return "world"
            end
        "#;

        let result = exec(&lua, "test_code", content);
        assert!(result.is_ok());

        let hello: mlua::Function = lua.globals().get("hello").unwrap();
        let result: String = hello.call(()).unwrap();
        assert_eq!(result, "world");
    }

    #[test]
    fn test_json() {
        let lua = Lua::new();
        assert!(
            set_loaded(&lua, "json", include_str!("json.lua"), LoadType::Table).is_ok(),
            "json test - load json.lua"
        );
        println!("Doing json test");
        let table = lua.create_table().unwrap();
        table.set("name", "John").unwrap();
        table.set("age", 30).unwrap();
        table.set("is_active", true).unwrap();
        assert!(
            lua.globals().set("users", table).is_ok(),
            "json test - setting global users table"
        );
        let content = r#"
            local json = require("json")
            return json.encode(users)
        "#;

        let result: LuaResult<String> = lua.load(content).eval();
        assert!(
            result.is_ok(),
            "json test - lua load code to json.encode table"
        );
        assert_eq!(
            result.unwrap(), r#"{"name":"John","age":30,"is_active":true}"#,
            "json test: encode lua table -> json string"
        );

        let json_string = r#"{"name":"Bob","age":96,"is_active":false}"#;
        assert!(
            lua.globals().set("users", json_string).is_ok(),
            "json test - setting global json_string"
        );
        let content = r#"
            local json = require('json')
            local users = json.decode(users)
            return users["name"]
        "#;

        let result: LuaResult<String> = lua.load(content).eval();
        assert_eq!(
            result.unwrap(), "Bob",
            "json test: decode json string -> lua table"
        );
    }

    #[test]
    fn test_stringify() {
        let lua = Lua::new();
        assert!(
            set_loaded(&lua, "json", include_str!("json.lua"), LoadType::Table).is_ok(),
            "stringify test - load json.lua"
        );
        assert!(
            set_loaded(&lua, ".stringify", include_str!("stringify.lua"), LoadType::Table).is_ok(),
            "stringify test - load stringify.lua"
        );
        let content = r#"
            local stringify = require(".stringify")
            local users_array = {{name = "Bob"}}
            return stringify.format(users_array)
        "#;
        let result: String = lua.load(content).eval().unwrap();
        let expected_result = "{\n  {\n     \u{1b}[31mname\u{1b}[0m = \u{1b}[32m\"Bob\"\u{1b}[0m\n  }\n }";
        assert_eq!(
            result, expected_result,
            "stringify test: formatted lua table -> json string"
        );
    }

    // This should probably be moved to the root tests folder to group with integration tests
    #[test]
    fn test_loader() -> LuaResult<()> {
        let lua = Lua::new();
        let globals = lua.globals();
        let package: LuaTable = globals.get("package")?;
        let loaded: LuaTable = package.get("loaded")?;

        lua.globals().set("println", lua.create_function(|_, s: String| { ao_log!("{}", s);Ok(()) }).unwrap()).unwrap();
        // let dump_src = include_str!("dump.lua");
        // let dump: LuaFunction = lua.load(dump_src).eval()?; //.map_err(|e| {
        // loaded.set(".dump", dump)?;

        // Imports for process.lua -- The order matters
        assert!(set_loaded(&lua, "ao", include_str!("ao.lua"), LoadType::Table).is_ok(), "\n** Error setting loaded: ao.lua");
        assert!(set_loaded(&lua, "json", include_str!("json.lua"), LoadType::Table).is_ok(), "\n** Error setting loaded: json.lua");
        assert!(set_loaded(&lua, ".dump", include_str!("dump.lua"), LoadType::Function).is_ok(), "\n** Error setting loaded: dump.lua");
        assert!(set_loaded(&lua, ".pretty", include_str!("pretty.lua"), LoadType::Table).is_ok(), "\n** Error setting loaded: pretty.lua");
        assert!(set_loaded(&lua, ".base64", include_str!("base64.lua"), LoadType::Table).is_ok(), "\n** Error setting loaded: base64.lua");
        assert!(set_loaded(&lua, ".chance", include_str!("chance.lua"), LoadType::Table).is_ok(), "\n** Error setting loaded: chance.lua");
        assert!(set_loaded(&lua, ".stringify", include_str!("stringify.lua"), LoadType::Table).is_ok(), "\n** Error setting loaded: stringify.lua");
        assert!(set_loaded(&lua, ".utils", include_str!("utils.lua"), LoadType::Table).is_ok(), "\n** Error setting loaded: utils.lua");
        assert!(set_loaded(&lua, ".handlers-utils", include_str!("handlers-utils.lua"), LoadType::Table).is_ok(), "\n** Error setting loaded: handlers-utils.lua");
        assert!(set_loaded(&lua, ".handlers", include_str!("handlers.lua"), LoadType::Table).is_ok(), "\n** Error setting loaded: handlers.lua");
        assert!(set_loaded(&lua, ".eval", include_str!("eval.lua"), LoadType::Function).is_ok(), "\n** Error setting loaded: eval.lua");
        // assert!(set_eval_lua(&lua).is_ok(), "\n** Error setting custom eval module");
        // assert!(exec(&lua, "main", include_str!("main.lua")).is_ok(), "\n** Error setting loaded: main.lua");
        assert!(set_loaded(&lua, ".default", include_str!("default.lua"), LoadType::Function).is_ok(), "\n** Error setting loaded: default.lua");
        // // loader.lua imports
        assert!(set_loaded(&lua, ".process", include_str!("process.lua"), LoadType::Table).is_ok(), "\n** Error setting loaded: process.lua");
        assert!(set_loaded(&lua, "handle", include_str!("loader.lua"), LoadType::Function).is_ok(), "\n****\n\t**  Error setting loaded: loader.lua\n****\n");

        let handle = loaded.get::<_, LuaFunction>("handle")?;
        let env = r#"
        {
            "Process": {
                "Id": "AOS",
                "Owner": "FOOBAR",
                "Tags": [{"name": "Name", "value": "Thomas"}]
            }
        }
        "#;
        let msg1 = r#"{
            "Target": "AOS",
            "Owner": "FOOBAR",
            "Block-Height": "1000",
            "Id": "1234xyxfoo",
            "Module": "WOOPAWOOPA",
            "Tags": [{"name": "Action", "value": "Eval"}],
            "Data": "Handlers.add('marcopolo', Handlers.utils.hasMatchingTag('Action', 'marco'), function(Msg) print('polo') end)"
        }"#;
        let result1 = handle.call::<_, String>((msg1, env))?;

        println!("Result 1 = {}", result1);

        let msg2 = r#"{
            "Target": "AOS",
            "Owner": "FOOBAR",
            "Block-Height": "1000",
            "Id": "1234xyxfoo",
            "Module": "WOOPAWOOPA",
            "Tags": [{"name": "Action", "value": "marco"}],
            "Data": ""
        }"#;
        // let result2 = handle.call::<_, String>((msg2, env))?;
        let result2: serde_json::Value = match handle.call::<_, String>((msg2, env)) {
            Ok(r) => match serde_json::from_str(&*r) {
                Ok(s) => s,
                Err(e) => return Err(LuaError::external(e.to_string()))
            },
            Err(e) => {
                println!("{}", e);
                return Err(e)
            }
        };
        println!("Result 2 = {}", result2);
        let data: String = match result2.get("response") {
            Some(response) => match response.get("Output") {
                Some(output) => match output.get("data") {
                    Some(data) => serde_json::to_string(data).unwrap_or_else(|_| "".to_string()),
                    None => "".to_string(),
                },
                None => "".to_string(),
            },
            None => "".to_string(),
        };
        println!("response data = {}", data);

        Ok(())
    }
}






// #[test]
// #[should_panic(expected = "Divide result is zero")]
// fn test_specific_panic() {
//     divide_non_zero_result(1, 10);
// }