//! XTask - Unified build system for ZakatRS
//!
//! Replaces platform-specific shell/PowerShell scripts with a cross-platform Rust task runner.
//!
//! # Usage
//! ```sh
//! cargo run -p xtask -- <command>
//! ```
//!
//! # Available Commands
//! - `build-all`     - Build all targets (Rust, Python, WASM, Dart)
//! - `sync-versions` - Synchronize versions across all package manifests
//! - `publish-all`   - Publish to all registries (interactive)
//! - `test`          - Run all tests

use anyhow::{bail, Context, Result};
use regex::Regex;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let task = &args[1];
    
    match task.as_str() {
        "build-all" => build_all()?,
        "sync-versions" => sync_versions()?,
        "publish-all" => publish_all()?,
        "test" => run_tests()?,
        "-h" | "--help" | "help" => print_usage(),
        _ => {
            eprintln!("❌ Unknown command: {}", task);
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!(
        r#"
🚀 ZakatRS XTask - Unified Build System

USAGE:
    cargo run -p xtask -- <COMMAND>
    cargo xtask <COMMAND>  (if .cargo/config.toml alias is set)

COMMANDS:
    build-all       Build all targets (Rust, Python, WASM, Dart)
    sync-versions   Synchronize versions across all package manifests
    publish-all     Publish to all registries (interactive)
    test            Run all tests

EXAMPLES:
    cargo run -p xtask -- build-all
    cargo run -p xtask -- sync-versions
"#
    );
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Get the project root directory (where root Cargo.toml lives)
fn project_root() -> Result<PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let root = PathBuf::from(manifest_dir)
        .parent()
        .context("Failed to find project root")?
        .to_path_buf();
    Ok(root)
}

/// Run a command and stream output in real-time
fn run_cmd(cmd: &str, args: &[&str]) -> Result<()> {
    println!("  → {} {}", cmd, args.join(" "));
    
    let status = Command::new(cmd)
        .args(args)
        .current_dir(project_root()?)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to execute: {} {}", cmd, args.join(" ")))?;

    if !status.success() {
        bail!("Command failed with exit code: {:?}", status.code());
    }
    Ok(())
}

/// Run a command in a specific directory
fn run_cmd_in_dir(dir: &Path, cmd: &str, args: &[&str]) -> Result<()> {
    println!("  → [{}] {} {}", dir.display(), cmd, args.join(" "));
    
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to execute: {} {} in {}", cmd, args.join(" "), dir.display()))?;

    if !status.success() {
        bail!("Command failed with exit code: {:?}", status.code());
    }
    Ok(())
}

/// Check if a command exists in PATH
fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

/// Copy a file from src to dst, creating parent directories if needed
fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(src, dst)?;
    println!("  ✅ Copied {} → {}", src.display(), dst.display());
    Ok(())
}

/// Copy a directory recursively
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    println!("  ✅ Copied directory {} → {}", src.display(), dst.display());
    Ok(())
}

/// Read the version from root Cargo.toml
fn read_cargo_version() -> Result<String> {
    let root = project_root()?;
    let cargo_path = root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)
        .with_context(|| format!("Failed to read {}", cargo_path.display()))?;
    
    let re = Regex::new(r#"version\s*=\s*"([^"]+)""#)?;
    let caps = re.captures(&content)
        .context("Could not find version in Cargo.toml")?;
    
    Ok(caps[1].to_string())
}

// =============================================================================
// Task: build-all
// =============================================================================

fn build_all() -> Result<()> {
    println!("\n🚀 Starting ZakatRS Master Build...\n");
    let root = project_root()?;

    // 0. Sync Versions
    println!("🔄 Synchronizing Versions...");
    sync_versions()?;

    // 1. Native Rust Build
    println!("\n🦀 Building Native Rust (Release)...");
    run_cmd("cargo", &["build", "--release"])?;

    // 2. Python Build (Maturin)
    println!("\n🐍 Building Python Package (Maturin)...");
    if command_exists("maturin") {
        run_cmd("maturin", &["build", "--release"])?;
    } else {
        println!("  ⚠️ 'maturin' not in PATH, trying 'python -m maturin'...");
        run_cmd("python", &["-m", "maturin", "build", "--release"])?;
    }

    // 3. WASM & JSR Build
    println!("\n🕸️  Building WASM & JSR Package...");
    if command_exists("wasm-pack") {
        build_wasm()?;
    } else {
        println!("  ⚠️ 'wasm-pack' not found! Skipping WASM build.");
    }

    // Always sync WASM/JS metadata
    sync_pkg_metadata()?;

    // 4. Dart/Flutter Prep
    println!("\n💙 Preparing Dart/Flutter Package...");
    build_dart(&root)?;

    println!("\n✅✅✅ ALL BUILDS COMPLETE! ✅✅✅");
    println!(" - Rust: target/release");
    println!(" - Python: target/wheels");
    println!(" - WASM/JS: pkg/");
    println!(" - Dart: zakat_dart/");

    Ok(())
}

fn build_wasm() -> Result<()> {
    let root = project_root()?;
    
    println!("  🏗️  Building WASM package...");
    run_cmd("wasm-pack", &["build", "--target", "nodejs", "--scope", "islamic"])?;
    
    println!("  📦 Restoring JSR configuration...");
    copy_file(&root.join("jsr-config/jsr.json"), &root.join("pkg/jsr.json"))?;
    copy_file(&root.join("jsr-config/mod.ts"), &root.join("pkg/mod.ts"))?;
    copy_file(&root.join("README.md"), &root.join("pkg/README.md"))?;
    copy_dir_recursive(&root.join("docs"), &root.join("pkg/docs"))?;
    
    println!("  ✅ WASM build complete!");
    Ok(())
}

fn sync_pkg_metadata() -> Result<()> {
    let root = project_root()?;
    
    println!("  📦 Syncing JS/WASM Metadata...");
    
    // Ensure pkg directory exists
    let pkg_dir = root.join("pkg");
    fs::create_dir_all(&pkg_dir)?;
    
    // Copy JSR Config
    copy_file(&root.join("jsr-config/jsr.json"), &root.join("pkg/jsr.json"))?;
    copy_file(&root.join("jsr-config/mod.ts"), &root.join("pkg/mod.ts"))?;
    
    // Copy Root Metadata
    copy_file(&root.join("README.md"), &root.join("pkg/README.md"))?;
    copy_file(&root.join("LICENSE"), &root.join("pkg/LICENSE"))?;
    
    // Copy Documentation
    copy_dir_recursive(&root.join("docs"), &root.join("pkg/docs"))?;
    
    println!("  ✅ pkg/ metadata synced.");
    Ok(())
}

fn build_dart(root: &Path) -> Result<()> {
    let dart_dir = root.join("zakat_dart");
    
    println!("  📦 Syncing Documentation to zakat_dart...");
    
    // Copy README
    copy_file(&root.join("README.md"), &dart_dir.join("README.md"))?;
    
    // Copy Docs (Renamed to 'doc' for Dart standard)
    let doc_dir = dart_dir.join("doc");
    copy_dir_recursive(&root.join("docs"), &doc_dir)?;
    
    // Copy License
    copy_file(&root.join("LICENSE"), &dart_dir.join("LICENSE"))?;
    
    // Copy Changelog
    copy_file(&root.join("CHANGELOG.md"), &dart_dir.join("CHANGELOG.md"))?;
    
    println!("  ✨ Dart package ready! Go to ./zakat_dart and run 'dart pub publish'");
    Ok(())
}

// =============================================================================
// Task: sync-versions
// =============================================================================

fn sync_versions() -> Result<()> {
    let root = project_root()?;
    let version = read_cargo_version()?;
    
    println!("🎯 Source Truth: Cargo.toml version is '{}'", version);
    
    // Update JSON files ("version": "x.y.z")
    update_json_version(&root.join("jsr-config/jsr.json"), &version)?;
    update_json_version(&root.join("pkg/jsr.json"), &version)?;
    update_json_version(&root.join("pkg/package.json"), &version)?;
    
    // Update Dart Pubspec (version: x.y.z)
    update_yaml_version(&root.join("zakat_dart/pubspec.yaml"), &version)?;
    
    // Update README.md (Dependency Examples)
    update_readme_versions(&root.join("README.md"), &version)?;
    
    // Update pyproject.toml if it has a static version
    update_pyproject_version(&root.join("pyproject.toml"), &version)?;
    
    println!("🔄 Version synchronization complete.");
    Ok(())
}

fn update_json_version(path: &Path, version: &str) -> Result<()> {
    if !path.exists() {
        println!("  - Skipping {} (not found)", path.display());
        return Ok(());
    }
    
    let content = fs::read_to_string(path)?;
    let re = Regex::new(r#""version"\s*:\s*"[^"]*""#)?;
    let new_content = re.replace(&content, format!(r#""version": "{}""#, version)).to_string();
    
    if content == new_content {
        println!("  - {} is already up to date.", path.display());
    } else {
        fs::write(path, &new_content)?;
        println!("  ✅ Updated {} to {}", path.display(), version);
    }
    Ok(())
}

fn update_yaml_version(path: &Path, version: &str) -> Result<()> {
    if !path.exists() {
        println!("  - Skipping {} (not found)", path.display());
        return Ok(());
    }
    
    let content = fs::read_to_string(path)?;
    let re = Regex::new(r"(?m)^version:\s*.*$")?;
    let new_content = re.replace(&content, format!("version: {}", version)).to_string();
    
    if content == new_content {
        println!("  - {} is already up to date.", path.display());
    } else {
        fs::write(path, &new_content)?;
        println!("  ✅ Updated {} to {}", path.display(), version);
    }
    Ok(())
}

fn update_readme_versions(path: &Path, version: &str) -> Result<()> {
    if !path.exists() {
        println!("  - Skipping {} (not found)", path.display());
        return Ok(());
    }
    
    let content = fs::read_to_string(path)?;
    
    // Pattern 1: zakat = "x.y.z"
    let re1 = Regex::new(r#"zakat\s*=\s*"[^"]*""#)?;
    let content = re1.replace_all(&content, format!(r#"zakat = "{}""#, version)).to_string();
    
    // Pattern 2: zakat = { version = "x.y.z"
    let re2 = Regex::new(r#"zakat\s*=\s*\{\s*version\s*=\s*"[^"]*""#)?;
    let new_content = re2.replace_all(&content, format!(r#"zakat = {{ version = "{}""#, version)).to_string();
    
    let original = fs::read_to_string(path)?;
    if original == new_content {
        println!("  - {} is already up to date.", path.display());
    } else {
        fs::write(path, &new_content)?;
        println!("  ✅ Updated {} to {}", path.display(), version);
    }
    Ok(())
}

fn update_pyproject_version(path: &Path, version: &str) -> Result<()> {
    if !path.exists() {
        println!("  - Skipping {} (not found)", path.display());
        return Ok(());
    }
    
    let content = fs::read_to_string(path)?;
    
    // Check if there's a static version entry
    let re = Regex::new(r#"version\s*=\s*"[^"]*""#)?;
    if !re.is_match(&content) {
        println!("  - {} uses dynamic version (skipped)", path.display());
        return Ok(());
    }
    
    let new_content = re.replace(&content, format!(r#"version = "{}""#, version)).to_string();
    
    if content == new_content {
        println!("  - {} is already up to date.", path.display());
    } else {
        fs::write(path, &new_content)?;
        println!("  ✅ Updated {} to {}", path.display(), version);
    }
    Ok(())
}

// =============================================================================
// Task: publish-all
// =============================================================================

fn publish_all() -> Result<()> {
    let root = project_root()?;
    
    println!("\n🚀 Starting ZakatRS Master Publish...");
    println!("⚠️  Checking prerequisites...");
    println!("   1. Have you bumped the version in Cargo.toml?");
    println!("   2. Have you run 'cargo xtask build-all'?");
    println!("   3. Are you logged in to all registries (cargo, npm, dart)?");
    println!();
    
    print!("Proceed with publishing to ALL repositories? (y/n) ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() != "y" {
        println!("Aborted.");
        return Ok(());
    }

    // 1. Rust (Crates.io)
    println!("\n🦀 Publishing to Crates.io...");
    run_cmd("cargo", &["publish"])?;

    // 2. Python (PyPI)
    println!("\n🐍 Publishing to PyPI...");
    if command_exists("maturin") {
        run_cmd("maturin", &["publish"])?;
    } else {
        run_cmd("python", &["-m", "maturin", "publish"])?;
    }

    // 3. NPM (JS)
    println!("\nnpm Publishing to NPM...");
    run_cmd_in_dir(&root.join("pkg"), "npm", &["publish", "--access", "public"])?;

    // 4. JSR (JS)
    println!("\n🦕 Publishing to JSR...");
    run_cmd_in_dir(&root.join("pkg"), "npx", &["jsr", "publish"])?;

    // 5. Dart (Pub.dev)
    println!("\n💙 Publishing to Pub.dev...");
    run_cmd_in_dir(&root.join("zakat_dart"), "dart", &["pub", "publish"])?;

    println!("\n✅✅✅ GLOBAL PUBLISH COMPLETE! ✅✅✅");
    Ok(())
}

// =============================================================================
// Task: test
// =============================================================================

fn run_tests() -> Result<()> {
    println!("\n🧪 Running all tests...\n");
    
    // Run Rust tests
    println!("🦀 Running Rust tests...");
    run_cmd("cargo", &["test"])?;
    
    println!("\n✅ All tests passed!");
    Ok(())
}
