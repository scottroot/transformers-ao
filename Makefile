all: clean build

.PHONY: build
build:
	docker run --rm \
		-v .:/src \
		scottroot/ao:latest \
		bash -c "\
			RUST_BACKTRACE=1 \
			TOKENIZERS_PARALLELISM=false \
			RAYON_RS_NUM_THREADS=1 \
			cargo build --release --target wasm32-unknown-emscripten"

.PHONY: clean
clean:
	cargo clean

.PHONY: build-docker-container
build-docker-container:
	docker build --platform linux/amd64 -t scottroot/ao:latest build/
