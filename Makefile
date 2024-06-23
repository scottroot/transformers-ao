all: clean build

# Versions align to Emscripten image versions.
CONTAINER_VERSION := latest

.PHONY: clean
clean:
	@files=$$(ls build/transformers_ao.* 2>/dev/null); \
	if [ -n "$$files" ]; then \
		file_list=$$(echo $$files | tr ' ' '\n'); \
		echo "Removing:"; \
		echo "$$file_list" | sed 's/^/ - /'; \
		rm $$files; \
	else \
		echo "No files to remove."; \
	fi
	cargo clean

.PHONY: build-container
build-container:
	docker build --platform linux/amd64 -t scottroot/ao:$(CONTAINER_VERSION) build/

.PHONY: build
build:
	docker run --rm \
		-v .:/src \
		scottroot/ao:$(CONTAINER_VERSION) \
		bash -c "RUST_BACKTRACE=1 CARGO_HOME=/src/.cargo/cache TOKENIZERS_PARALLELISM=false RAYON_RS_NUM_THREADS=1 \
			cargo build --release --target wasm32-unknown-emscripten"
	@echo
	@echo "Patching glue code (patch-emscripten-gluecode.mjs)" && node build/patch-emscripten-gluecode.mjs build/transformers_ao.js
	@echo "Wasm Exports:" && wasm2wat build/transformers_ao.wasm | grep "  (export" | grep -v "dynCall" | sort
	@echo "Wasm Imports:" && wasm2wat build/transformers_ao.wasm | grep "  (import" | grep -v "invoke_" | sort
	@echo "Wasm ObjDump:" && wasm-objdump -h build/transformers_ao.wasm
	@echo "Finished"

CONTAINER64_VERSION := 3.1.61
.PHONY: build64-container
build64-container:
	cd build/rust64 && ./run.sh

.PHONY: build64
build64:
	docker run --rm -it -v .:/src scottroot/ao64:$(CONTAINER64_VERSION) bash -c "\
			RUST_BACKTRACE=1 \
			TOKENIZERS_PARALLELISM=false \
			RAYON_RS_NUM_THREADS=1 \
			cargo build -Zbuild-std --release --target wasm64-unknown-unknown"
