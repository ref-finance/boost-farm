RFLAGS="-C link-arg=-s"

build: contracts/boost-farming
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p meme-farming --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/meme_farming.wasm ./res/meme_farming.wasm

unittest: build
ifdef TC
	RUSTFLAGS=$(RFLAGS) cargo test $(TC) -p meme-farming --lib -- --nocapture
else
	RUSTFLAGS=$(RFLAGS) cargo test -p meme-farming --lib -- --nocapture
endif

test: build mock-ft mock-mft
ifdef TF
	RUSTFLAGS=$(RFLAGS) cargo test -p meme-farming --test $(TF) -- --nocapture
else
	RUSTFLAGS=$(RFLAGS) cargo test -p meme-farming --tests
endif

release:
	$(call docker_build,_rust_setup.sh)
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/meme_farming.wasm res/meme_farming_release.wasm


mock-ft: contracts/mock-ft
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-ft --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_ft.wasm ./res/mock_ft.wasm

mock-mft: contracts/mock-mft
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p mock-mft --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/mock_mft.wasm ./res/mock_mft.wasm

clean:
	cargo clean
	rm -rf res/

define docker_build
	docker build -t my-meme-farm-builder .
	docker run \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host \
		-e RUSTFLAGS=$(RFLAGS) \
		-i -t my-meme-farm-builder \
		/bin/bash $(1)
endef
