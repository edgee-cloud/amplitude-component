.PHONY: all
MAKEFLAGS += --silent

all: help

help:
	@grep -E '^[a-zA-Z1-9\._-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| sort \
		| sed -e "s/^Makefile://" -e "s///" \
		| awk 'BEGIN { FS = ":.*?## " }; { printf "\033[36m%-30s\033[0m %s\n", $$1, $$2 }'

dev.setup: ## Install dependencies
	test -s wasi_snapshot_preview1.reactor.wasm || \
	curl -LO https://github.com/bytecodealliance/wasmtime/releases/download/v23.0.1/wasi_snapshot_preview1.reactor.wasm

dev.build: ## Build the wasi component
	cargo build --target wasm32-wasip1
	wasm-tools component \
		new ./target/wasm32-wasip1/debug/amplitude_component.wasm \
		-o amplitude.wasm \
		--adapt wasi_snapshot_preview1.reactor.wasm

ci.setup: ## Install dependencies
	test -s wasi_snapshot_preview1.reactor.wasm || \
		nix develop --command \
			curl -LO https://github.com/bytecodealliance/wasmtime/releases/download/v23.0.1/wasi_snapshot_preview1.reactor.wasm

ci.build: ## Build the wasi component
	nix develop --command cargo build --release --target wasm32-wasip1
	nix develop --command wasm-tools component \
		new ./target/wasm32-wasip1/release/amplitude_component.wasm \
		-o amplitude.wasm \
		--adapt wasi_snapshot_preview1.reactor.wasm