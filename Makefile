.PHONY: all
all: clean build

.PHONY: build
build:
	LUA_LIB=./build/lua-5.3.4/src cargo build --release --target wasm32-unknown-emscripten

.PHONY: clean
clean:
	rm -rf target/
	rm -f build/compile.c
	rm -f build/lua-5.3.4/src/*.wasm
