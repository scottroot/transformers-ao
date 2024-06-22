use std;
use std::process::Command;


fn main() {
    // COMPILE WEAVEDRIVE
    let weavedrive_src = "src/weavedrive.c";
    let weavedrive_outfile = "build/weavedrive.o";
    let weavedrive_status = Command::new("emcc")
        .args(&[
            "-s", "WASM=1",
            "-s", "SUPPORT_LONGJMP=1",
            "-c", weavedrive_src,
            "-o", &weavedrive_outfile,
        ])
        .status()
        .expect("Failed to compile Weavedrive C code with emcc");

    if !weavedrive_status.success() {
        panic!("emcc failed to compile the C source file");
    }
}