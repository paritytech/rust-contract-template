#!/usr/bin/env bash

set -euo pipefail

cd "${0%/*}/"
RUSTC_BOOTSTRAP=1 cargo build --release
polkatool link --strip --output contract.polkavm target/riscv64emac-unknown-none-polkavm/release/contract
