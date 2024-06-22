use mlua::Lua;
use mlua::prelude::{LuaResult, LuaTable};

#[cfg(not(test))]
#[link(wasm_import_module = "env")]
extern "C" {
    #[link_name = "__asyncjs__weavedrive_open"]
    fn weavedrive_open(c_filename: *const i8, mode: *const i8) -> i32;
    #[link_name = "__asyncjs__weavedrive_read"]
    fn weavedrive_read(fd: i32, dst_ptr: *mut i8, length: usize) -> i32;
    // I thought there was a close function, but need to check again
    // fn weavedrive_close(fd: i32) -> i32;
}


#[cfg(all(test, target_os = "linux"))]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::{Read, Write};
#[cfg(test)]
fn weavedrive_open(c_filename: *const i8, mode: *const i8) -> i32 {
    let filename = unsafe { std::ffi::CStr::from_ptr(c_filename).to_str().unwrap() };
    let mode = unsafe { std::ffi::CStr::from_ptr(mode).to_str().unwrap() };
    let file = match mode {
        "r" => std::fs::File::open(filename),
        "w" => std::fs::File::create(filename),
        _ => return -1,
    };

    match file {
        Ok(file) => file.as_raw_fd(),
        Err(_) => -1,
    }
}

#[cfg(test)]
fn weavedrive_read(fd: i32, dst_ptr: *mut i8, length: usize) -> i32 {
    let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    let mut buffer = vec![0; length];
    let bytes_read = match file.read(&mut buffer) {
        Ok(bytes) => bytes,
        Err(_) => return -1,
    };
    unsafe {
        std::ptr::copy_nonoverlapping(buffer.as_ptr(), dst_ptr as *mut u8, bytes_read);
    }
    std::mem::forget(file);
    bytes_read as i32
}

#[cfg(test)]
fn weavedrive_close(_fd: i32) -> i32 {
    // let mut file = unsafe { std::fs::File::from_raw_fd(fd) };
    // drop(file);
    0
}


#[cfg(not(test))]
pub fn open(filename: &str, mode: &str) -> i32 {
    let c_filename = std::ffi::CString::new(filename).unwrap();
    let c_mode = std::ffi::CString::new(mode).unwrap();
    unsafe { weavedrive_open(c_filename.as_ptr(), c_mode.as_ptr()) }
}

#[cfg(test)]
pub fn open(filename: &str, mode: &str) -> i32 {
    let c_filename = std::ffi::CString::new(filename).unwrap();
    let c_mode = std::ffi::CString::new(mode).unwrap();
    weavedrive_open(c_filename.as_ptr(), c_mode.as_ptr())
}

#[cfg(not(test))]
pub fn read(fd: i32, buffer: &mut [u8]) -> i32 {
    unsafe { weavedrive_read(fd, buffer.as_mut_ptr() as *mut i8, buffer.len()) }
}
#[cfg(test)]
pub fn read(fd: i32, buffer: &mut [u8]) -> i32 {
    weavedrive_read(fd, buffer.as_mut_ptr() as *mut i8, buffer.len())
}

pub fn close(_fd: i32) -> i32 {
    // unsafe { weavedrive_close(fd) }
    0
}

pub fn preload(lua: &Lua) -> LuaResult<()> {
    let wd_table = lua.create_table()?;
    wd_table.set("_version", "0.0.1")?;

    let open = lua.create_function(|_, (filename, mode): (String, Option<String>)| {
        let mode = mode.unwrap_or_else(|| "r".to_string());
        let fd = open(&filename, &mode);
        if fd == 0 {
            return Ok(None);
        }
        Ok(Some(fd))
    })?;
    wd_table.set("open", open)?;

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
    wd_table.set("read", read)?;

    let close = lua.create_function(|_, fd: i32| {
        close(fd);
        Ok(())
    })?;
    wd_table.set("close", close)?;

    let package: LuaTable = lua.globals().get("package")?;
    let loaded: LuaTable = package.get("loaded")?;
    loaded.set("weavedrive", wd_table)?;

    Ok(())
}