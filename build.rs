use anyhow::{Context, Result};
use std::{env, fs, path::PathBuf, process::Command};

fn main() -> Result<()> {
    // Prevent infinite recursion when building the contract itself
    if env::var("SKIP_BUILD_SCRIPT").is_ok() {
        return Ok(());
    }

    // Setup paths
    let manifest_dir: PathBuf = env::var("CARGO_MANIFEST_DIR")?.into();
    let out_dir: PathBuf = env::var("OUT_DIR")?.into();

    // Rerun if contract source changes
    println!("cargo::rerun-if-changed=src/main.rs");

    // Build the contract to RISC-V ELF
    let build_dir = out_dir.join("pvm-build");
    fs::create_dir_all(&build_dir)?;

    invoke_build(&manifest_dir, &build_dir)?;

    // Link ELF to PolkaVM bytecode
    let elf_path = build_dir.join("target/riscv64emac-unknown-none-polkavm/release/contract");

    link_to_polkavm(&elf_path)?;

    Ok(())
}

fn invoke_build(manifest_dir: &PathBuf, build_dir: &PathBuf) -> Result<()> {
    let encoded_rustflags = ["-Dwarnings"].join("\x1f");

    let mut args = polkavm_linker::TargetJsonArgs::default();
    args.is_64_bit = true;

    let mut build_command = Command::new("cargo");
    build_command
        .current_dir(manifest_dir)
        .env_clear()
        .env("PATH", env::var("PATH").unwrap_or_default())
        .env("CARGO_ENCODED_RUSTFLAGS", encoded_rustflags)
        .env("CARGO_TARGET_DIR", build_dir.join("target"))
        .env("RUSTUP_HOME", env::var("RUSTUP_HOME").unwrap_or_default())
        .env("RUSTC_BOOTSTRAP", "1")
        .env("SKIP_BUILD_SCRIPT", "1")
        .args([
            "build",
            "--release",
            "-Zbuild-std=core,alloc",
            "-Zbuild-std-features=panic_immediate_abort",
        ])
        .arg("--target")
        .arg(polkavm_linker::target_json_path(args).map_err(|e| anyhow::anyhow!(e))?);

    let build_res = build_command
        .output()
        .context("Failed to execute cargo build")?;

    if !build_res.status.success() {
        let stderr = String::from_utf8_lossy(&build_res.stderr);
        eprintln!("{}", stderr);
        anyhow::bail!("Failed to build contract");
    }

    Ok(())
}

fn link_to_polkavm(elf_path: &PathBuf) -> Result<()> {
    let mut config = polkavm_linker::Config::default();
    config.set_strip(true);
    config.set_optimize(true);

    let elf_bytes =
        fs::read(elf_path).with_context(|| format!("Failed to read ELF from {:?}", elf_path))?;

    let linked = polkavm_linker::program_from_elf(
        config,
        polkavm_linker::TargetInstructionSet::ReviveV1,
        &elf_bytes,
    )
    .map_err(|err| anyhow::anyhow!("Failed to link PolkaVM program: {}", err))?;

    let output_path = "./contract.polkavm";
    fs::write(output_path, linked)
        .with_context(|| format!("Failed to write PolkaVM bytecode to {:?}", output_path))?;

    Ok(())
}
