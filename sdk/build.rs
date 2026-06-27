//! Derives `DOPPLER_BINARY_SIZE` from the deploy artifact at compile time, building the
//! program first (and installing platform-tools) if it is missing.

use std::{env, fs, path::PathBuf, process::Command};

/// platform-tools pin; keep in sync with run-validator.sh.
const TOOLS_VERSION: &str = "v1.54";

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace = manifest_dir.parent().unwrap();
    let deploy_dir = workspace.join("target/deploy");
    let so = deploy_dir.join("doppler_program.so");

    // Track the artifact so the size follows an external rebuild.
    println!("cargo:rerun-if-changed={}", so.display());

    if !so.exists() {
        println!("cargo:warning=doppler_program.so missing; building it via cargo build-sbf");

        // Install the pinned platform-tools if absent.
        let tools_present = env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(".cache/solana").join(TOOLS_VERSION).exists())
            .unwrap_or(false);
        if !tools_present {
            println!("cargo:warning=platform-tools {TOOLS_VERSION} missing; installing");
            let installed = Command::new("cargo")
                .args(["build-sbf", "--install-only", "--tools-version", TOOLS_VERSION])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            assert!(installed, "failed to install platform-tools {TOOLS_VERSION}");
        }

        // Separate CARGO_TARGET_DIR so the nested build can't deadlock on the outer lock.
        let status = Command::new("cargo")
            .arg("build-sbf")
            .args(["--tools-version", TOOLS_VERSION])
            .args(["--manifest-path", workspace.join("program/Cargo.toml").to_str().unwrap()])
            .args(["--sbf-out-dir", deploy_dir.to_str().unwrap()])
            .env("CARGO_TARGET_DIR", workspace.join("target/sbf-build"))
            .status()
            .expect("failed to run `cargo build-sbf`");
        assert!(status.success(), "`cargo build-sbf` failed");
    }

    let len = fs::metadata(&so)
        .unwrap_or_else(|e| panic!("{} unavailable after build: {e}", so.display()))
        .len();

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("doppler_binary_size.rs");
    fs::write(out, format!("pub(crate) const DOPPLER_BINARY_SIZE: u32 = {len};\n")).unwrap();
}
