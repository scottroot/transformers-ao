use mlua::Lua;
use mlua::prelude::{LuaResult, LuaTable};


#[link(wasm_import_module = "env")]
extern "C" {
    #[link_name = "__asyncjs__weavedrive_open"]
    fn weavedrive_open(c_filename: *const i8, mode: *const i8) -> i32;
    #[link_name = "__asyncjs__weavedrive_read"]
    fn weavedrive_read(fd: i32, dst_ptr: *mut i8, length: usize) -> i32;
    // I thought there was a close function, but need to check again
    // fn weavedrive_close(fd: i32) -> i32;
}

pub fn open(filename: &str, mode: &str) -> i32 {
    let c_filename = std::ffi::CString::new(filename).unwrap();
    let c_mode = std::ffi::CString::new(mode).unwrap();
    unsafe { weavedrive_open(c_filename.as_ptr(), c_mode.as_ptr()) }
}

pub fn read(fd: i32, buffer: &mut [u8]) -> i32 {
    unsafe { weavedrive_read(fd, buffer.as_mut_ptr() as *mut i8, buffer.len()) }
}

pub fn close(_fd: i32) -> i32 {
    // unsafe { weavedrive_close(fd) }
    0
}

pub fn preload(lua: &Lua) -> LuaResult<()> {
    let io_table = lua.create_table()?;
    io_table.set("_version", "0.0.1")?;

    let open = lua.create_function(|_, (filename, mode): (String, Option<String>)| {
        let mode = mode.unwrap_or_else(|| "r".to_string());
        let fd = open(&filename, &mode);
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
            let bytes_read = read(fd, &mut temp_buffer);
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
        close(fd);
        Ok(())
    })?;
    io_table.set("close", close)?;

    let package: LuaTable = lua.globals().get("package")?;
    let loaded: LuaTable = package.get("loaded")?;
    loaded.set("weavedrive", io_table)?;

    Ok(())
}