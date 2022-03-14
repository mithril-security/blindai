XARGO_PATH != realpath '../sgx_sdk/xargo'

check: 
	RUSTFLAGS="-Z force-unstable-if-unmarked" RUST_TARGET_PATH=$(XARGO_PATH) xargo check --target x86_64-mesalock-unknown-sgx

.PHONY: check