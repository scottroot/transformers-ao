# Build Script
## Compile Rust + Lua to wasm32-unknown-emscripten

#### Maintains same/similar functionality to the emcc-lua Python compiler script from:
* the old [ysugimoto/webassembly-lua repo](https://github.com/ysugimoto/webassembly-lua)
* which can also be found in the [AO repo](https://github.com/permaweb/ao/blob/main/dev-cli/container/Dockerfile)

#### -- Here it is bolstered by Rust's strong typing.

#### Script currently only scans the local lua file directory (where your custom Lua scripts and other AOS files would be) and includes .lua files (not library .so or .a files).  Will revisit this if a need arises for using other libs or a LuaRocks dependency.

#### Compiling to wasm64-unknown-unknown is also possible.
Can compile to whatever rustup toolchain is available. In this script, for now, it is just conditionally running if the target is wasm32-unknown-emscripten, however at some point will maybe test wasm64-unknown-unknown if there is a reason for it and the AOLoader provides compatibility with it.
