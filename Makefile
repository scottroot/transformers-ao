all: clean build

.PHONY: clean build build-docker-container build-custom-libc

#CONTAINER_VERSION := 3.1.61
CONTAINER_VERSION := latest

EMCC_CACHE := build/libc_objects/cache/sysroot/lib/wasm32-emscripten
build:
	@-rm outtest.*
	docker run --rm \
		-v .:/src \
		scottroot/ao:$(CONTAINER_VERSION) \
		bash -c "\
			RUST_BACKTRACE=1 \
			TOKENIZERS_PARALLELISM=false \
			RAYON_RS_NUM_THREADS=1 \
			CARGO_HOME=/src/.cargo/cache \
			cargo build --release --target wasm32-unknown-emscripten"
	@echo
	@echo "Patching glue code (format-loader.mjs)" && node tests/ao-loader/format-loader.mjs build/transformers_ao.js
	@echo "Wasm Exports:" && wasm2wat build/transformers_ao.wasm | grep "  (export" | grep -v "dynCall" | sort
	@echo "Wasm Imports:" && wasm2wat build/transformers_ao.wasm | grep "  (import" | grep -v "invoke_" | sort
	@echo "Wasm ObjDump:" && wasm-objdump -h build/transformers_ao.wasm

.PHONY: build64
build64:
	docker run --rm -it -v .:/src scottroot/ao64:latest bash -c "\
			RUST_BACKTRACE=1 \
			TOKENIZERS_PARALLELISM=false \
			RAYON_RS_NUM_THREADS=1 \
			cargo build -Zbuild-std --release --target wasm64-unknown-unknown"

build64-container:
	cd build/rust64 && ./run.sh


build-custom-libc:
	-rm build/aolibc/libc.a
	cd build/aolibc && ./make-extension.sh
	cp build/aolibc/aolibc.a build/aolibc/libc.a

EMSDK_PATH ?= $(shell echo $$EMSDK)
LIBC_DEBUG_A_PATH ?= $(shell find $(EMSDK_PATH) -name libc-debug.a)

build-custom-libc-debug:
	echo "EMSDK Path = $(EMSDK_PATH)"
	-rm build/libc-debug.a
	mkdir -p build/libc-debug
	cd build/libc-debug && emar x $(LIBC_DEBUG_A_PATH) && rm fopen.o fread.o fclose.o && emar cr libc-debug.a *.o
	mv build/libc-debug/libc-debug.a $(EMCC_CACHE)/libc-debug.a
	echo $(shell $($(EMCC_CACHE)/libc-debug.a))

clean:
	cargo clean

build-docker-container:
	docker build --platform linux/amd64 -t scottroot/ao:$(CONTAINER_VERSION) build/

.PHONY: test
test:
	node tests/manual.test.js
	#cargo test --features lib -- --nocapture

.PHONY: objdump
objdump:
	docker run --rm -it -v .:/src scottroot/ao:latest llvm-objdump -h outtest.wasm