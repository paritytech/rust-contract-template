TARGETS = all clean
.PHONY: $(TARGETS)
.SILENT: $(TARGETS)

all:
	# RUSTC_BOOTSTRAP is required in order to use unstable features
	RUSTC_BOOTSTRAP=1 cargo build --release
	polkatool link --strip --output contract.polkavm target/riscv64emac-unknown-none-polkavm/release/contract

clean:
	cargo clean
