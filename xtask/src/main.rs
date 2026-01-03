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
        "publish-crates" => publish_crates()?,
        "publish-pypi" => publish_pypi()?,
        "publish-npm" => publish_npm()?,
        "publish-jsr" => publish_jsr()?,
        "publish-dart" => publish_dart()?,
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
    cargo run -p xtask -- <COMMAND> [OPTIONS]
    cargo xtask <COMMAND>  (if .cargo/config.toml alias is set)

COMMANDS:
    build-all       Build all targets (Rust, Python, WASM, Dart)
    sync-versions   Synchronize versions across all package manifests
    test            Run all tests

PUBLISH COMMANDS:
    publish-all     Publish to all registries (interactive)
    publish-crates  Publish to Crates.io only
    publish-pypi    Publish to PyPI only
    publish-npm     Publish to NPM only
    publish-jsr     Publish to JSR only
    publish-dart    Publish to Pub.dev only

PUBLISH OPTIONS:
    --dry-run, -n   Validate without actually publishing
    --skip-crates   Skip crates.io publishing (publish-all only)
    --skip-pypi     Skip PyPI publishing (publish-all only)
    --skip-npm      Skip NPM publishing (publish-all only)
    --skip-jsr      Skip JSR publishing (publish-all only)
    --skip-dart     Skip pub.dev publishing (publish-all only)

EXAMPLES:
    cargo xtask build-all
    cargo xtask sync-versions
    cargo xtask publish-all --dry-run
    cargo xtask publish-dart --dry-run
    cargo xtask publish-npm
    cargo xtask publish-all --skip-dart --skip-pypi
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
        .stdin(Stdio::inherit())
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
        .stdin(Stdio::inherit())
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

/// Generate type definitions for all supported languages using typeshare.
/// 
/// Generates:
/// - TypeScript: `pkg/types.ts` (for NPM, JSR, WASM)
/// - Kotlin: `zakat_android/lib/src/main/java/com/islamic/zakat/Types.kt` (for Android)
/// - Swift: `zakat_ios/Sources/ZakatTypes.swift` (for iOS - future)
/// 
/// Note: Dart is not supported by typeshare. For Dart/Flutter, types are
/// generated via `flutter_rust_bridge` which reads the Rust source directly.
fn generate_types() -> Result<()> {
    println!("📝 Generating type definitions for all platforms...");
    
    // Check if typeshare-cli is installed
    if !command_exists("typeshare") {
        println!("  ⚠️ 'typeshare' CLI not found. Installing via cargo...");
        run_cmd("cargo", &["install", "typeshare-cli"])?;
    }
    
    let root = project_root()?;
    let zakat_core_path = root.join("zakat-core");
    let input_str = zakat_core_path.to_string_lossy().to_string();

    // === TypeScript (for NPM, JSR, WASM) ===
    println!("\n  🟦 Generating TypeScript types...");
    let ts_output = root.join("pkg").join("types.ts");
    fs::create_dir_all(root.join("pkg"))?;
    
    run_cmd("typeshare", &[
        &input_str,
        "--lang=typescript",
        &format!("--output-file={}", ts_output.to_string_lossy()),
    ])?;
    println!("    ✅ TypeScript: pkg/types.ts");

    // === Kotlin (for Android) ===
    println!("\n  🟩 Generating Kotlin types...");
    let kotlin_dir = root.join("zakat_android").join("lib").join("src")
        .join("main").join("java").join("com").join("islamic").join("zakat");
    fs::create_dir_all(&kotlin_dir)?;
    let kotlin_output = kotlin_dir.join("Types.kt");
    
    run_cmd("typeshare", &[
        &input_str,
        "--lang=kotlin",
        "--java-package=com.islamic.zakat",
        &format!("--output-file={}", kotlin_output.to_string_lossy()),
    ])?;
    println!("    ✅ Kotlin: zakat_android/.../Types.kt");

    // === Swift (for iOS - optional, create directory if needed) ===
    let swift_dir = root.join("zakat_ios").join("Sources");
    if swift_dir.exists() || root.join("zakat_ios").exists() {
        println!("\n  🟧 Generating Swift types...");
        fs::create_dir_all(&swift_dir)?;
        let swift_output = swift_dir.join("ZakatTypes.swift");
        
        run_cmd("typeshare", &[
            &input_str,
            "--lang=swift",
            "--swift-prefix=Zakat",
            &format!("--output-file={}", swift_output.to_string_lossy()),
        ])?;
        println!("    ✅ Swift: zakat_ios/Sources/ZakatTypes.swift");
    } else {
        println!("\n  🟧 Swift: Skipped (zakat_ios/ directory not found)");
    }

    println!("\n  ✅ All type definitions generated!");
    println!("  ℹ️  Note: Dart types are generated by flutter_rust_bridge, not typeshare.");
    Ok(())
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

    // 1. Generate TypeScript Types from Rust
    generate_types()?;

    // 2. Native Rust Build
    println!("\n🦀 Building Native Rust (Release)...");
    run_cmd("cargo", &["build", "--release"])?;

    // 3. Python Build (Maturin)
    println!("\n🐍 Building Python Package (Maturin)...");
    let zakat_manifest = root.join("zakat").join("Cargo.toml");
    let manifest_arg = format!("-m={}", zakat_manifest.display());
    
    if command_exists("maturin") {
        run_cmd("maturin", &["build", "--release", &manifest_arg])?;
    } else {
        println!("  ⚠️ 'maturin' not in PATH, trying 'python -m maturin'...");
        run_cmd("python", &["-m", "maturin", "build", "--release", &manifest_arg])?;
    }

    // 4. WASM & JSR Build
    println!("\n🕸️  Building WASM & JSR Package...");
    if command_exists("wasm-pack") {
        build_wasm()?;
    } else {
        println!("  ⚠️ 'wasm-pack' not found! Skipping WASM build.");
    }

    // Always sync WASM/JS metadata
    sync_pkg_metadata()?;

    // 5. Dart/Flutter Prep
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
    let zakat_dir = root.join("zakat");
    
    println!("  🏗️  Building WASM package...");
    run_cmd_in_dir(&zakat_dir, "wasm-pack", &[
        "build", 
        "--target", "nodejs", 
        "--scope", "islamic",
        "--out-dir", root.join("pkg").to_string_lossy().as_ref(),
    ])?;
    
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
    
    // Note: types.ts is already generated directly to pkg/types.ts by generate_types()
    // No need to copy it from jsr-config
    
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

/// Workspace crates in dependency order (leaves first, root last)
const WORKSPACE_CRATES: &[&str] = &[
    "zakat-core",      // No internal deps
    "zakat-i18n",      // Depends on zakat-core
    "zakat-ledger",    // Depends on zakat-core
    "zakat-providers", // Depends on zakat-core
    "zakat-sqlite",    // Depends on zakat-core, zakat-ledger
    "zakat",           // Facade - depends on all above
];

fn publish_all() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    let skip_crates = args.iter().any(|a| a == "--skip-crates");
    let skip_pypi = args.iter().any(|a| a == "--skip-pypi");
    let skip_npm = args.iter().any(|a| a == "--skip-npm");
    let skip_jsr = args.iter().any(|a| a == "--skip-jsr");
    let skip_dart = args.iter().any(|a| a == "--skip-dart");
    
    let root = project_root()?;
    let version = read_cargo_version()?;
    
    println!("\n🚀 ZakatRS Master Publish v{}", version);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if dry_run {
        println!("🔍 DRY RUN MODE - No actual publishing will occur\n");
    }
    
    println!("📋 Publish targets:");
    println!("   • Crates.io: {} crates {}", WORKSPACE_CRATES.len(), if skip_crates { "(SKIPPED)" } else { "" });
    println!("   • PyPI: zakatrs {}", if skip_pypi { "(SKIPPED)" } else { "" });
    println!("   • NPM: @islamic/zakat {}", if skip_npm { "(SKIPPED)" } else { "" });
    println!("   • JSR: @islam/zakat {}", if skip_jsr { "(SKIPPED)" } else { "" });
    println!("   • pub.dev: zakat {}", if skip_dart { "(SKIPPED)" } else { "" });
    println!();
    
    if !dry_run {
        println!("⚠️  Prerequisites:");
        println!("   1. Version bumped in Cargo.toml");
        println!("   2. 'cargo xtask build-all' completed successfully");
        println!("   3. Logged in to: cargo, pypi, npm, jsr, dart pub");
        println!();
        
        print!("Proceed with publishing? (y/n) ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() != "y" {
            println!("❌ Aborted.");
            return Ok(());
        }
    }

    let mut success_count = 0;
    let mut fail_count = 0;

    // 1. Rust (Crates.io) - Publish each crate in dependency order
    if !skip_crates {
        println!("\n🦀 Publishing to Crates.io ({} crates)...", WORKSPACE_CRATES.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        for (i, crate_name) in WORKSPACE_CRATES.iter().enumerate() {
            println!("\n  [{}/{}] Publishing {}...", i + 1, WORKSPACE_CRATES.len(), crate_name);
            
            let crate_dir = root.join(crate_name);
            if !crate_dir.exists() {
                println!("    ⚠️  Directory not found: {}", crate_dir.display());
                fail_count += 1;
                continue;
            }
            
            let result = if dry_run {
                run_cmd_in_dir(&crate_dir, "cargo", &["publish", "--dry-run"])
            } else {
                run_cmd_in_dir(&crate_dir, "cargo", &["publish"])
            };
            
            match result {
                Ok(_) => {
                    println!("    ✅ {} published successfully!", crate_name);
                    success_count += 1;
                }
                Err(e) => {
                    println!("    ❌ Failed to publish {}: {}", crate_name, e);
                    fail_count += 1;
                    
                    if !dry_run {
                        print!("    Continue with remaining crates? (y/n) ");
                        io::stdout().flush()?;
                        let mut input = String::new();
                        io::stdin().read_line(&mut input)?;
                        if input.trim().to_lowercase() != "y" {
                            bail!("Publishing aborted by user after {} failure", crate_name);
                        }
                    }
                }
            }
        }
    }

    // 2. Python (PyPI)
    if !skip_pypi {
        println!("\n🐍 Publishing to PyPI...");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let zakat_crate = root.join("zakat");
        let manifest_arg = format!("-m={}", zakat_crate.join("Cargo.toml").display());
        
        let result = if dry_run {
            if command_exists("maturin") {
                run_cmd("maturin", &["build", "--release", &manifest_arg])
            } else {
                run_cmd("python", &["-m", "maturin", "build", "--release", &manifest_arg])
            }
        } else {
            if command_exists("maturin") {
                run_cmd("maturin", &["publish", &manifest_arg])
            } else {
                run_cmd("python", &["-m", "maturin", "publish", &manifest_arg])
            }
        };
        
        match result {
            Ok(_) => {
                println!("  ✅ PyPI {} successful!", if dry_run { "dry-run" } else { "publish" });
                success_count += 1;
            }
            Err(e) => {
                println!("  ❌ PyPI failed: {}", e);
                fail_count += 1;
            }
        }
    }

    // 3. NPM
    if !skip_npm {
        println!("\n📦 Publishing to NPM...");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let result = if dry_run {
            run_cmd_in_dir(&root.join("pkg"), "npm", &["publish", "--access", "public", "--dry-run"])
        } else {
            run_cmd_in_dir(&root.join("pkg"), "npm", &["publish", "--access", "public"])
        };
        
        match result {
            Ok(_) => {
                println!("  ✅ NPM {} successful!", if dry_run { "dry-run" } else { "publish" });
                success_count += 1;
            }
            Err(e) => {
                println!("  ❌ NPM failed: {}", e);
                fail_count += 1;
            }
        }
    }

    // 4. JSR
    if !skip_jsr {
        println!("\n🦕 Publishing to JSR...");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let result = if dry_run {
            run_cmd_in_dir(&root.join("pkg"), "npx", &["jsr", "publish", "--dry-run"])
        } else {
            run_cmd_in_dir(&root.join("pkg"), "npx", &["jsr", "publish"])
        };
        
        match result {
            Ok(_) => {
                println!("  ✅ JSR {} successful!", if dry_run { "dry-run" } else { "publish" });
                success_count += 1;
            }
            Err(e) => {
                println!("  ❌ JSR failed: {}", e);
                fail_count += 1;
            }
        }
    }

    // 5. Dart (Pub.dev)
    if !skip_dart {
        println!("\n💙 Publishing to Pub.dev...");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let result = if dry_run {
            run_cmd_in_dir(&root.join("zakat_dart"), "dart", &["pub", "publish", "--dry-run"])
        } else {
            run_cmd_in_dir(&root.join("zakat_dart"), "dart", &["pub", "publish", "--force"])
        };
        
        match result {
            Ok(_) => {
                println!("  ✅ Pub.dev {} successful!", if dry_run { "dry-run" } else { "publish" });
                success_count += 1;
            }
            Err(e) => {
                println!("  ❌ Pub.dev failed: {}", e);
                fail_count += 1;
            }
        }
    }

    // Summary
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if fail_count == 0 {
        println!("✅✅✅ ALL PUBLISH OPERATIONS {}! ✅✅✅", if dry_run { "VALIDATED" } else { "COMPLETE" });
    } else {
        println!("⚠️  Publish completed with {} success, {} failures", success_count, fail_count);
    }
    
    if dry_run {
        println!("\n💡 To actually publish, run without --dry-run:");
        println!("   cargo run -p xtask -- publish-all");
    }

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

// =============================================================================
// Individual Publish Tasks
// =============================================================================

/// Publish only to Crates.io
fn publish_crates() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    
    let root = project_root()?;
    let version = read_cargo_version()?;
    
    println!("\n🦀 Publishing to Crates.io v{}", version);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if dry_run {
        println!("🔍 DRY RUN MODE - No actual publishing will occur\n");
    }
    
    println!("📋 Publishing {} crates in dependency order:", WORKSPACE_CRATES.len());
    for crate_name in WORKSPACE_CRATES {
        println!("   • {}", crate_name);
    }
    println!();
    
    if !dry_run {
        print!("Proceed with publishing? (y/n) ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() != "y" {
            println!("❌ Aborted.");
            return Ok(());
        }
    }

    let mut success_count = 0;
    let mut fail_count = 0;
    
    for (i, crate_name) in WORKSPACE_CRATES.iter().enumerate() {
        println!("\n  [{}/{}] Publishing {}...", i + 1, WORKSPACE_CRATES.len(), crate_name);
        
        let crate_dir = root.join(crate_name);
        if !crate_dir.exists() {
            println!("    ⚠️  Directory not found: {}", crate_dir.display());
            fail_count += 1;
            continue;
        }
        
        let result = if dry_run {
            run_cmd_in_dir(&crate_dir, "cargo", &["publish", "--dry-run"])
        } else {
            run_cmd_in_dir(&crate_dir, "cargo", &["publish"])
        };
        
        match result {
            Ok(_) => {
                println!("    ✅ {} published successfully!", crate_name);
                success_count += 1;
            }
            Err(e) => {
                println!("    ❌ Failed to publish {}: {}", crate_name, e);
                fail_count += 1;
                
                if !dry_run {
                    print!("    Continue with remaining crates? (y/n) ");
                    io::stdout().flush()?;
                    let mut input = String::new();
                    io::stdin().read_line(&mut input)?;
                    if input.trim().to_lowercase() != "y" {
                        bail!("Publishing aborted by user after {} failure", crate_name);
                    }
                }
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if fail_count == 0 {
        println!("✅ Crates.io publish {}!", if dry_run { "validated" } else { "complete" });
    } else {
        println!("⚠️  {} success, {} failures", success_count, fail_count);
    }
    
    Ok(())
}

/// Publish only to PyPI
fn publish_pypi() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    
    let root = project_root()?;
    let version = read_cargo_version()?;
    
    println!("\n🐍 Publishing to PyPI v{}", version);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if dry_run {
        println!("🔍 DRY RUN MODE - Will only build, not publish\n");
    }
    
    let zakat_crate = root.join("zakat");
    let manifest_arg = format!("-m={}", zakat_crate.join("Cargo.toml").display());
    
    let result = if dry_run {
        if command_exists("maturin") {
            run_cmd("maturin", &["build", "--release", &manifest_arg])
        } else {
            run_cmd("python", &["-m", "maturin", "build", "--release", &manifest_arg])
        }
    } else {
        if command_exists("maturin") {
            run_cmd("maturin", &["publish", &manifest_arg])
        } else {
            run_cmd("python", &["-m", "maturin", "publish", &manifest_arg])
        }
    };
    
    match result {
        Ok(_) => println!("\n✅ PyPI {} successful!", if dry_run { "dry-run" } else { "publish" }),
        Err(e) => println!("\n❌ PyPI failed: {}", e),
    }
    
    Ok(())
}

/// Publish only to NPM
fn publish_npm() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    
    let root = project_root()?;
    let version = read_cargo_version()?;
    
    println!("\n📦 Publishing to NPM v{}", version);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if dry_run {
        println!("🔍 DRY RUN MODE - No actual publishing will occur\n");
    }
    
    let pkg_dir = root.join("pkg");
    if !pkg_dir.exists() {
        bail!("pkg/ directory not found. Run 'cargo xtask build-all' first.");
    }
    
    let result = if dry_run {
        run_cmd_in_dir(&pkg_dir, "npm", &["publish", "--access", "public", "--dry-run"])
    } else {
        run_cmd_in_dir(&pkg_dir, "npm", &["publish", "--access", "public"])
    };
    
    match result {
        Ok(_) => println!("\n✅ NPM {} successful!", if dry_run { "dry-run" } else { "publish" }),
        Err(e) => println!("\n❌ NPM failed: {}", e),
    }
    
    Ok(())
}

/// Publish only to JSR
fn publish_jsr() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    
    let root = project_root()?;
    let version = read_cargo_version()?;
    
    println!("\n🦕 Publishing to JSR v{}", version);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if dry_run {
        println!("🔍 DRY RUN MODE - No actual publishing will occur\n");
    }
    
    let pkg_dir = root.join("pkg");
    if !pkg_dir.exists() {
        bail!("pkg/ directory not found. Run 'cargo xtask build-all' first.");
    }
    
    let result = if dry_run {
        run_cmd_in_dir(&pkg_dir, "npx", &["jsr", "publish", "--dry-run"])
    } else {
        run_cmd_in_dir(&pkg_dir, "npx", &["jsr", "publish"])
    };
    
    match result {
        Ok(_) => println!("\n✅ JSR {} successful!", if dry_run { "dry-run" } else { "publish" }),
        Err(e) => println!("\n❌ JSR failed: {}", e),
    }
    
    Ok(())
}

/// Publish only to Pub.dev (Dart)
fn publish_dart() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    
    let root = project_root()?;
    let version = read_cargo_version()?;
    
    println!("\n💙 Publishing to Pub.dev v{}", version);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if dry_run {
        println!("🔍 DRY RUN MODE - No actual publishing will occur\n");
    }
    
    let dart_dir = root.join("zakat_dart");
    if !dart_dir.exists() {
        bail!("zakat_dart/ directory not found.");
    }
    
    let result = if dry_run {
        run_cmd_in_dir(&dart_dir, "dart", &["pub", "publish", "--dry-run"])
    } else {
        run_cmd_in_dir(&dart_dir, "dart", &["pub", "publish", "--force"])
    };
    
    match result {
        Ok(_) => println!("\n✅ Pub.dev {} successful!", if dry_run { "dry-run" } else { "publish" }),
        Err(e) => println!("\n❌ Pub.dev failed: {}", e),
    }
    
    Ok(())
}
