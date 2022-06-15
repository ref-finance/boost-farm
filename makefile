RFLAGS="-C link-arg=-s"

build: contracts/boost-farming
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p boost-farming --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/boost_farming.wasm ./res/boost_farming.wasm

test: build mock-ft mock-mft
	RUSTFLAGS=$(RFLAGS) cargo test -p boost-farming

release:
	$(call docker_build,_rust_setup.sh)
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/boost_farming.wasm res/boost_farming_release.wasm

TEST_FILE ?= **
LOGS ?=
sandbox: build mock-ft mock-mft
	cp res/*.wasm sandbox/compiled-contracts/
	cd sandbox && \
	NEAR_PRINT_LOGS=$(LOGS) npx near-workspaces-ava --timeout=5m __tests__/boost-farming/$(TEST_FILE).ava.ts --verbose

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
	docker build -t my-contract-builder .
	docker run \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		-w /host \
		-e RUSTFLAGS=$(RFLAGS) \
		-i -t my-contract-builder \
		/bin/bash $(1)
endef
