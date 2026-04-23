# Build the on-chain verifier script for CKB-VM
TARGET := riscv64imac-unknown-none-elf
CONTRACT_DIR := contracts

.PHONY: build clean test

build:
	cargo build --target $(TARGET) --release -p ckb-ultraplonk-verifier
	@mkdir -p $(CONTRACT_DIR)
	cp target/$(TARGET)/release/ckb-ultraplonk-verifier $(CONTRACT_DIR)/

clean:
	cargo clean
	rm -rf $(CONTRACT_DIR)/*

test: build
	cargo test -p ckb-ultraplonk-verifier-tests
