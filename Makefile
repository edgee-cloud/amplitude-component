.PHONY: all
MAKEFLAGS += --silent

all: help

help:
	@grep -E '^[a-zA-Z1-9\._-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| sort \
		| sed -e "s/^Makefile://" -e "s///" \
		| awk 'BEGIN { FS = ":.*?## " }; { printf "\033[36m%-30s\033[0m %s\n", $$1, $$2 }'

install: ## Install dependencies
	test -s wasi_snapshot_preview1.reactor.wasm || \
	curl -LO https://github.com/bytecodealliance/wasmtime/releases/download/v25.0.2/wasi_snapshot_preview1.reactor.wasm

build: ## Build the wasi component
	cargo build --target wasm32-wasip1
	wasm-tools component \
		new ./target/wasm32-wasip1/debug/amplitude_component.wasm \
		-o amplitude.wasm \
		--adapt wasi_snapshot_preview1.reactor.wasm