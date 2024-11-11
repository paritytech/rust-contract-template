# Contract Rust Template

This is a minimal template for a Rust contract targeting [`pallet_revive`](https://contracts.polkadot.io). This is a low-
level way to write contracts, so we don't expect it to be used for implementing high-level contract logic. Instead, we expect
that Rust will be used to implement libraries that are then called by Solidity, similar to Python, where performance-critical
code is written in C.

## Components

In terms of code, this template is very bare bones. `main.rs` is just a few lines of code. Most of the files in this repo
deal with compiling the code to PolkaVM in a `rust-analyzer`-friendly way. We included a `rust-toolchain.toml` and a
`.cargo/config.toml` so that all tools automatically select the correct target and toolchain (we need a relatively new `nightly`).

The `call_from_sol.sol` file demonstrates how to call the example in `main.rs` from Solidity.

## Memory Allocation

The contract depends on the `pallet-revive-uapi` crate, which is a thin (but safe) wrapper around all available host functions. It only
includes the absolute minimum. This means we also don't include a memory allocator. If you want to use `alloc`, you need to define
a global allocator. Note that we don't support dynamic memory allocations in `pallet_revive` yet. Therefore, the allocator would need
to operate on a static buffer.

## How to Build

You can build this project with `cargo build`. However, to generate a valid contract, you also need to link it. Linking means taking the
ELF file outputted by the Rust compiler and transforming it into a PolkaVM module.

```sh
# This makes sure that polkatool is on the newest version
$ cargo install polkatool

# This will build the project and then use polkatool to link it
$ ./build.sh
```

**The build result is placed as `contract.polkavm` in the repository root. This is the final artifact that can be deployed as-is.**

## How to Inspect the Contract

```sh
$ polkatool stats contract.polkavm
$ polkatool disassemble contract.polkavm
```

## Examples

The [test fixtures](https://github.com/paritytech/polkadot-sdk/tree/master/substrate/frame/revive/fixtures/contracts) for `pallet_revive` are
written in the same way as this template and might be useful as examples.
