#!/usr/bin/env bash

set -euo pipefail

cd "${0%/*}/"
cargo build --release
polkatool link --strip --output contract.polkavm target/riscv32emac-unknown-none-polkavm/release/contract
