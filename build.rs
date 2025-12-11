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

    // Rerun if contract sources change
    println!("cargo::rerun-if-changed=src/contract_with_alloc.rs");
    println!("cargo::rerun-if-changed=src/contract_no_alloc.rs");

    invoke_build(&manifest_dir, &out_dir, "contract_with_alloc")?;
    let elf_path_with_alloc =
        out_dir.join("target/riscv64emac-unknown-none-polkavm/release/contract_with_alloc");
    link_to_polkavm(&elf_path_with_alloc, "contract_with_alloc.polkavm")?;

    invoke_build(&manifest_dir, &out_dir, "contract_no_alloc")?;
    let elf_path_no_alloc =
        out_dir.join("target/riscv64emac-unknown-none-polkavm/release/contract_no_alloc");
    link_to_polkavm(&elf_path_no_alloc, "contract_no_alloc.polkavm")?;

    Ok(())
}

fn invoke_build(manifest_dir: &PathBuf, build_dir: &PathBuf, bin_name: &str) -> Result<()> {
    let mut args = polkavm_linker::TargetJsonArgs::default();
    args.is_64_bit = true;

    let mut build_command = Command::new("cargo");
    build_command
        .current_dir(manifest_dir)
        .env_clear()
        .env("PATH", env::var("PATH").unwrap_or_default())
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
        .arg("--bin")
        .arg(bin_name)
        .arg("--target")
        .arg(polkavm_linker::target_json_path(args).map_err(|e| anyhow::anyhow!(e))?);

    let build_res = build_command
        .output()
        .context("Failed to execute cargo build")?;

    if !build_res.status.success() {
        let stderr = String::from_utf8_lossy(&build_res.stderr);
        eprintln!("{}", stderr);
        anyhow::bail!("Failed to build contract {}", bin_name);
    }

    Ok(())
}

fn link_to_polkavm(elf_path: &PathBuf, output_filename: &str) -> Result<()> {
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

    let output_path = format!("./{}", output_filename);
    fs::write(&output_path, linked)
        .with_context(|| format!("Failed to write PolkaVM bytecode to {:?}", output_path))?;

    Ok(())
}
