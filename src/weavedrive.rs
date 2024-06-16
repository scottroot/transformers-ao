#[link(wasm_import_module = "env")]
extern "C" {
    #[link_name = "__asyncjs__weavedrive_open"]
    fn weavedrive_open(c_filename: *const i8, mode: *const i8) -> i32;
    #[link_name = "__asyncjs__weavedrive_read"]
    fn weavedrive_read(fd: i32, dst_ptr: *mut i8, length: usize) -> i32;
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
    // I thought there was a close function, but need to check again
    0
}