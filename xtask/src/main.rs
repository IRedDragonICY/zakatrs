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
        "build-wasm" => build_wasm()?,
        "build-go" => build_go()?,
        "sync-versions" => sync_versions()?,
        "publish-all" => publish_all(None)?,
        "publish-crates" => publish_all(Some("crates"))?,
        "publish-pypi" => publish_all(Some("pypi"))?,
        "publish-npm" => publish_all(Some("npm"))?,
        "publish-jsr" => publish_all(Some("jsr"))?,
        "publish-dart" => publish_all(Some("dart"))?,
        "test" => run_tests()?,
        "test-all" => run_all_tests()?,
        "gen-test-suite" => generate_test_suite()?,
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
    build-wasm      Build WASM target only
    build-go        Build Go bindings via UniFFI
    sync-versions   Synchronize versions across all package manifests
    test            Run Rust tests only
    test-all        Run full compliance test suite (Rust + Python + Dart + WASM)
    gen-test-suite  Generate zakat_suite.json golden data

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
    cargo xtask test-all
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
        .with_context(|| format!("Failed to start command: {} {}", cmd, args.join(" ")))?;

    if !status.success() {
        bail!(
            "Command '{}' failed with exit code: {}\n    See output above for details.",
            cmd,
            status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())
        );
    }
    Ok(())
}

/// Run a command in a specific directory with output capture for better error messages
fn run_cmd_in_dir(dir: &Path, cmd: &str, args: &[&str]) -> Result<()> {
    println!("  → [{}] {} {}", dir.display(), cmd, args.join(" "));
    
    // On Windows, use cmd /C for shell commands (flutter.bat, npm.cmd, etc.)
    #[cfg(windows)]
    let output = Command::new("cmd")
        .args(["/C", cmd])
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("Failed to start command: {} {}", cmd, args.join(" ")))?;
    
    #[cfg(not(windows))]
    let output = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("Failed to start command: {} {}", cmd, args.join(" ")))?;

    // Always print stdout
    if !output.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    
    // Always print stderr
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if !output.status.success() {
        bail!(
            "Command '{}' failed with exit code: {}",
            cmd,
            output.status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())
        );
    }
    Ok(())
}

/// Run a command in a specific directory with live output (for interactive commands)
fn run_cmd_in_dir_interactive(dir: &Path, cmd: &str, args: &[&str]) -> Result<()> {
    println!("  → [{}] {} {}", dir.display(), cmd, args.join(" "));
    
    // On Windows, use cmd /C for shell commands like npx, npm, etc.
    #[cfg(windows)]
    let status = Command::new("cmd")
        .args(["/C", cmd])
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to start command: {} {}", cmd, args.join(" ")))?;
    
    #[cfg(not(windows))]
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to start command: {} {}", cmd, args.join(" ")))?;

    if !status.success() {
        bail!(
            "Command '{}' failed with exit code: {}\n    See output above for details.",
            cmd,
            status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string())
        );
    }
    Ok(())
}

/// Result of a publish attempt
#[derive(Debug)]
enum PublishResult {
    Success,
    AlreadyExists,
    Failed(String),
}

/// Run a command with retry logic for transient failures (502, 503, 429, etc.)
/// Useful for network-bound operations like cargo publish
/// Returns PublishResult to handle "already exists" gracefully
fn run_cmd_in_dir_with_retry_publish(dir: &Path, cmd: &str, args: &[&str], max_retries: u32) -> PublishResult {
    use std::thread;
    use std::time::Duration;
    
    let retry_delays = [5, 15, 30]; // seconds between retries
    
    for attempt in 0..=max_retries {
        if attempt > 0 {
            let delay = retry_delays.get((attempt - 1) as usize).copied().unwrap_or(30);
            println!("    ⏳ Retry {}/{} in {} seconds...", attempt, max_retries, delay);
            thread::sleep(Duration::from_secs(delay));
        }
        
        println!("  → [{}] {} {}", dir.display(), cmd, args.join(" "));
        
        #[cfg(windows)]
        let output = match Command::new("cmd")
            .args(["/C", cmd])
            .args(args)
            .current_dir(dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output() {
                Ok(o) => o,
                Err(e) => return PublishResult::Failed(format!("Failed to start command: {}", e)),
            };
        
        #[cfg(not(windows))]
        let output = match Command::new(cmd)
            .args(args)
            .current_dir(dir)
            .stdin(Stdio::inherit())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output() {
                Ok(o) => o,
                Err(e) => return PublishResult::Failed(format!("Failed to start command: {}", e)),
            };
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        // Always print output
        if !output.stdout.is_empty() {
            print!("{}", stdout);
        }
        if !output.stderr.is_empty() {
            eprint!("{}", stderr);
        }
        
        if output.status.success() {
            return PublishResult::Success;
        }
        
        let combined_output = format!("{}{}", stdout, stderr);
        
        // Check if crate already exists (not an error, just skip)
        if combined_output.contains("already exists") {
            return PublishResult::AlreadyExists;
        }
        
        // Check if error is retryable (transient network errors)
        let is_retryable = combined_output.contains("502")
            || combined_output.contains("503")
            || combined_output.contains("429")
            || combined_output.contains("Bad Gateway")
            || combined_output.contains("Service Unavailable")
            || combined_output.contains("rate limit")
            || combined_output.contains("timeout")
            || combined_output.contains("connection reset")
            || combined_output.contains("CloudFront");
        
        if is_retryable && attempt < max_retries {
            println!("    ⚠️  Transient error detected (attempt {}/{})", attempt + 1, max_retries + 1);
            continue;
        }
        
        // Non-retryable error or max retries exceeded
        return PublishResult::Failed(format!(
            "Command '{}' failed with exit code: {} (after {} attempts)",
            cmd,
            output.status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string()),
            attempt + 1
        ));
    }
    
    PublishResult::Failed("Max retries exceeded".to_string())
}

/// Check if a command exists in PATH
/// On Windows, we need to use cmd /C or check with `where`
fn command_exists(cmd: &str) -> bool {
    #[cfg(windows)]
    {
        // On Windows, use `where` command to check if executable exists
        Command::new("where")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        Command::new(cmd)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok()
    }
}

/// Check if a Python module is available via `python -m module --version`
fn python_module_exists(module: &str) -> bool {
    let python = get_venv_python();
    Command::new(&python)
        .args(["-m", module, "--version"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get the Python executable path, preferring venv if available
fn get_venv_python() -> String {
    let root = project_root().unwrap_or_else(|_| PathBuf::from("."));
    
    #[cfg(windows)]
    let venv_python = root.join(".venv").join("Scripts").join("python.exe");
    #[cfg(not(windows))]
    let venv_python = root.join(".venv").join("bin").join("python");
    
    if venv_python.exists() {
        venv_python.to_string_lossy().to_string()
    } else {
        "python".to_string()
    }
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
        "--",
        "--features", "wasm",
    ])?;
    
    println!("  📦 Restoring JSR configuration...");
    copy_file(&root.join("jsr-config/jsr.json"), &root.join("pkg/jsr.json"))?;
    copy_file(&root.join("jsr-config/mod.ts"), &root.join("pkg/mod.ts"))?;
    copy_file(&root.join("README.md"), &root.join("pkg/README.md"))?;
    copy_dir_recursive(&root.join("docs"), &root.join("pkg/docs"))?;
    
    println!("  ✅ WASM build complete!");
    Ok(())
}

// =============================================================================
// Task: build-go
// =============================================================================

/// Build Go bindings via UniFFI
///
/// This function:
/// 1. Builds zakat-core as a cdylib with uniffi feature
/// 2. Runs uniffi-bindgen-go to generate Go bindings
/// 3. Copies the generated files and library to zakat_go/
fn build_go() -> Result<()> {
    println!("\n🐹 Building Go Bindings via UniFFI...\n");
    let root = project_root()?;
    let zakat_core_dir = root.join("zakat-core");
    let go_dir = root.join("zakat_go");
    
    // Ensure zakat_go directory exists
    fs::create_dir_all(&go_dir)?;
    
    // Step 1: Build zakat-core as cdylib with uniffi feature
    println!("  🏗️  Building zakat-core as cdylib...");
    run_cmd_in_dir(&zakat_core_dir, "cargo", &[
        "build", 
        "--release", 
        "--features", "uniffi",
    ])?;
    
    // Determine library name based on platform
    #[cfg(target_os = "windows")]
    let lib_name = "zakat_core.dll";
    #[cfg(target_os = "macos")]
    let lib_name = "libzakat_core.dylib";
    #[cfg(target_os = "linux")]
    let lib_name = "libzakat_core.so";
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let lib_name = "libzakat_core.so"; // Fallback
    
    let lib_src = root.join("target").join("release").join(lib_name);
    let lib_dst = go_dir.join(lib_name);
    
    // Step 2: Check for uniffi-bindgen-go
    println!("  🔧 Checking for uniffi-bindgen-go...");
    if !command_exists("uniffi-bindgen-go") {
        println!("  ⚠️  uniffi-bindgen-go not found in PATH.");
        println!("     Install it with:");
        println!("       go install github.com/ArcticIcePak/uniffi-bindgen-go/bindgen@latest");
        println!("     Or from NordSecurity:");
        println!("       git clone https://github.com/NordSecurity/uniffi-bindgen-go");
        println!("       cd uniffi-bindgen-go && go install ./bindgen");
        println!();
        println!("  📦 Skipping binding generation. Copying library only...");
        
        // Still copy the library
        if lib_src.exists() {
            copy_file(&lib_src, &lib_dst)?;
            println!("  ✅ Library copied: {}", lib_dst.display());
        } else {
            println!("  ⚠️  Library not found: {}", lib_src.display());
        }
        
        println!("\n  ⚠️  Go build partially complete - bindings not generated.");
        println!("  💡 After installing uniffi-bindgen-go, run this command again.");
        return Ok(());
    }
    
    // Step 3: Generate Go bindings
    println!("  🔄 Generating Go bindings...");
    
    // Find the .udl file or use the library directly
    let udl_path = zakat_core_dir.join("src").join("zakat_core.udl");
    
    if udl_path.exists() {
        // Generate from UDL file
        run_cmd_in_dir(&go_dir, "uniffi-bindgen-go", &[
            udl_path.to_string_lossy().as_ref(),
            "--library", lib_src.to_string_lossy().as_ref(),
            "--out-dir", ".",
        ])?;
    } else {
        // Generate from library directly (scaffolding mode)
        println!("  📝 No .udl file found, generating from library scaffolding...");
        run_cmd_in_dir(&go_dir, "uniffi-bindgen-go", &[
            "--library", lib_src.to_string_lossy().as_ref(),
            "--out-dir", ".",
        ])?;
    }
    
    // Step 4: Copy the library
    println!("  📦 Copying dynamic library...");
    if lib_src.exists() {
        copy_file(&lib_src, &lib_dst)?;
    } else {
        println!("  ⚠️  Library not found at: {}", lib_src.display());
        println!("     Build may have failed or produced a different output.");
    }
    
    // Step 5: Copy documentation
    println!("  📚 Syncing documentation...");
    copy_file(&root.join("README.md"), &go_dir.join("README.md"))?;
    copy_file(&root.join("LICENSE"), &go_dir.join("LICENSE"))?;
    
    println!("\n  ✅ Go build complete!");
    println!("  📁 Output: zakat_go/");
    println!("  💡 To test: cd zakat_go && go test -v ./...");
    
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
    
    // Format Dart code for pub.dev static analysis
    // Only format lib, test, example - NOT cargokit submodule
    println!("  🎨 Formatting Dart code...");
    if command_exists("dart") {
        if let Err(e) = run_cmd_in_dir(&dart_dir, "dart", &["format", "lib", "test", "example", "integration_test"]) {
            println!("    ⚠️  dart format failed (non-fatal): {}", e);
        } else {
            println!("    ✅ Dart code formatted!");
        }
    } else {
        println!("    ⚠️  dart not found, skipping format");
    }
    
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
    "zakat-providers", // Depends on zakat-core, used by zakat-ledger
    "zakat-ledger",    // Depends on zakat-core, zakat-providers
    "zakat-sqlite",    // Depends on zakat-core, zakat-ledger
    "zakat-cli",       // Interactive CLI tool
    "zakat",           // Facade - depends on all above
];

struct RegistryTarget<'a> {
    id: &'a str,
    name: &'a str,
    skip_flag: &'a str,
    path_relative: &'a str,
    cmd: &'a str,
    args_publish: Vec<&'a str>,
    args_dry_run: Vec<&'a str>,
    is_interactive: bool,
}

fn get_registry_targets() -> Vec<RegistryTarget<'static>> {
    // Determine Dart command: prefer 'dart', fallback to 'flutter'
    let (dart_cmd, dart_args_pub, dart_args_dry) = if command_exists("dart") {
        ("dart", vec!["pub", "publish", "--force"], vec!["pub", "publish", "--dry-run"])
    } else if command_exists("flutter") {
        println!("  ⚠️  'dart' command not found, falling back to 'flutter pub publish'");
        ("flutter", vec!["pub", "publish", "--force"], vec!["pub", "publish", "--dry-run"])
    } else {
        // Fallback to dart and let it fail if neither exists
        ("dart", vec!["pub", "publish", "--force"], vec!["pub", "publish", "--dry-run"])
    };

    vec![
        RegistryTarget {
            id: "crates",
            name: "Crates.io",
            skip_flag: "--skip-crates",
            path_relative: ".",
            cmd: "cargo",
            args_publish: vec!["publish"],
            args_dry_run: vec!["publish", "--dry-run"],
            is_interactive: false,
        },
        RegistryTarget {
            id: "pypi",
            name: "PyPI",
            skip_flag: "--skip-pypi",
            path_relative: ".",
            cmd: "maturin",
            args_publish: vec!["publish"],
            args_dry_run: vec!["build", "--release"],
            is_interactive: false,
        },
        RegistryTarget {
            id: "npm",
            name: "NPM",
            skip_flag: "--skip-npm",
            path_relative: "pkg",
            cmd: "npm",
            args_publish: vec!["publish", "--access", "public"],
            args_dry_run: vec!["publish", "--access", "public", "--dry-run"],
            is_interactive: true,
        },
        RegistryTarget {
            id: "jsr",
            name: "JSR",
            skip_flag: "--skip-jsr",
            path_relative: "pkg",
            cmd: "npx",
            args_publish: vec!["jsr", "publish"],
            args_dry_run: vec!["jsr", "publish", "--dry-run"],
            is_interactive: true,
        },
        RegistryTarget {
            id: "dart",
            name: "Pub.dev",
            skip_flag: "--skip-dart",
            path_relative: "zakat_dart",
            cmd: dart_cmd,
            args_publish: dart_args_pub,
            args_dry_run: dart_args_dry,
            is_interactive: true,
        },
    ]
}

fn publish_all(specific_target: Option<&str>) -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    let skip_confirm = args.iter().any(|a| a == "--yes" || a == "-y") || env::var("CI").is_ok();
    
    let root = project_root()?;
    let version = read_cargo_version()?;
    
    let targets = get_registry_targets();
    
    // Filter targets
    let active_targets: Vec<&RegistryTarget> = targets.iter().filter(|t| {
        if let Some(specific) = specific_target {
            return t.id == specific;
        }
        !args.iter().any(|a| a == t.skip_flag)
    }).collect();

    if active_targets.is_empty() {
        println!("⚠️  No targets selected for publishing.");
        return Ok(());
    }

    // Header
    if specific_target.is_some() {
        let target = active_targets[0];
        println!("\n🚀 Publishing to {} v{}", target.name, version);
    } else {
        println!("\n🚀 ZakatRS Master Publish v{}", version);
    }
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    if dry_run {
        println!("🔍 DRY RUN MODE - No actual publishing will occur\n");
    }
    
    if specific_target.is_none() {
        println!("📋 Publish targets:");
        for t in &targets {
             let skipped = !active_targets.iter().any(|at| at.id == t.id);
             let status = if skipped { "(SKIPPED)" } else { "" };
             if t.id == "crates" {
                 println!("   • {}: {} crates {}", t.name, WORKSPACE_CRATES.len(), status);
             } else if t.id == "pypi" {
                 println!("   • {}: zakatrs {}", t.name, status);
             } else if t.id == "npm" {
                 println!("   • {}: @islamic/zakat {}", t.name, status);
             } else if t.id == "jsr" {
                 println!("   • {}: @islam/zakat {}", t.name, status);
             } else if t.id == "dart" {
                 println!("   • {}: zakat {}", t.name, status);
             }
        }
        println!();
        
        if !dry_run {
            println!("⚠️  Prerequisites:");
            println!("   1. Version bumped in Cargo.toml");
            println!("   2. 'cargo xtask build-all' completed successfully");
            println!("   3. Logged in to: cargo, pypi, npm, jsr, dart pub");
            println!();
        }
    } else if !dry_run && active_targets[0].id == "crates" {
         println!("📋 Publishing {} crates in dependency order:", WORKSPACE_CRATES.len());
         for crate_name in WORKSPACE_CRATES {
             println!("   • {}", crate_name);
         }
         println!();
    }

    if !dry_run && !skip_confirm {
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

    for target in active_targets {
        let emoji = match target.id {
            "crates" => "🦀",
            "pypi" => "🐍",
            "npm" => "📦",
            "jsr" => "🦕",
            "dart" => "💙",
            _ => "🚀",
        };
        
        println!("\n{} Publishing to {}...", emoji, target.name);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if target.id == "crates" {
             for (i, crate_name) in WORKSPACE_CRATES.iter().enumerate() {
                println!("\n  [{}/{}] Publishing {}...", i + 1, WORKSPACE_CRATES.len(), crate_name);
                let crate_dir = root.join(crate_name);
                if !crate_dir.exists() {
                    println!("    ⚠️  Directory not found: {}", crate_dir.display());
                    fail_count += 1;
                    continue;
                }
                
                if dry_run {
                    match run_cmd_in_dir(&crate_dir, "cargo", &["publish", "--dry-run"]) {
                        Ok(_) => {
                            println!("    ✅ {} dry-run passed!", crate_name);
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("    ❌ {} dry-run failed: {}", crate_name, e);
                            fail_count += 1;
                        }
                    }
                } else {
                    // Use retry with "already exists" detection
                    match run_cmd_in_dir_with_retry_publish(&crate_dir, "cargo", &["publish"], 3) {
                        PublishResult::Success => {
                            println!("    ✅ {} published successfully!", crate_name);
                            success_count += 1;
                        }
                        PublishResult::AlreadyExists => {
                            println!("    ⏭️  {} already exists, skipping.", crate_name);
                            success_count += 1; // Count as success since it's not a failure
                        }
                        PublishResult::Failed(e) => {
                            println!("    ❌ Failed to publish {}: {}", crate_name, e);
                            fail_count += 1;
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
        } else if target.id == "pypi" {
            let zakat_crate = root.join("zakat");
            let manifest_arg = format!("-m={}", zakat_crate.join("Cargo.toml").display());
            
            let args = if dry_run { target.args_dry_run.clone() } else { target.args_publish.clone() };
            let args_strings: Vec<String> = args.iter().map(|s| s.to_string()).collect();
            let mut final_args: Vec<&str> = args_strings.iter().map(|s| s.as_str()).collect();
            final_args.push(&manifest_arg);

            let result = if command_exists("maturin") {
                run_cmd("maturin", &final_args)
            } else {
                let mut python_args = vec!["-m", "maturin"];
                python_args.extend(final_args);
                run_cmd("python", &python_args)
            };
            
            match result {
                Ok(_) => {
                    println!("  ✅ {} {} successful!", target.name, if dry_run { "dry-run" } else { "publish" });
                    success_count += 1;
                }
                Err(e) => {
                    println!("  ❌ {} failed: {}", target.name, e);
                    fail_count += 1;
                }
            }
        } else if target.id == "npm" && !dry_run {
            // Special handling for NPM with auto-login on auth failure
            let dir = root.join(target.path_relative);
            if !dir.exists() {
                println!("    ⚠️  Directory not found: {}", dir.display());
                fail_count += 1;
                continue;
            }
            
            let args = &target.args_publish;
            let result = run_cmd_in_dir_interactive(&dir, target.cmd, args);
            
            match result {
                Ok(_) => {
                    println!("  ✅ {} publish successful!", target.name);
                    success_count += 1;
                }
                Err(_e) => {
                    // On any NPM failure, try re-login and retry
                    // Common issues: token expired, revoked, permission denied
                    println!("  ⚠️  NPM publish failed. Attempting re-authentication...");
                    
                    // Prompt for npm login
                    println!("  🔐 Running 'npm login' to refresh authentication...");
                    if run_cmd_in_dir_interactive(&dir, "npm", &["login"]).is_ok() {
                        println!("  🔄 Retrying publish after login...");
                        match run_cmd_in_dir_interactive(&dir, target.cmd, args) {
                            Ok(_) => {
                                println!("  ✅ {} publish successful after re-login!", target.name);
                                success_count += 1;
                            }
                            Err(e2) => {
                                println!("  ❌ {} still failed after re-login: {}", target.name, e2);
                                fail_count += 1;
                            }
                        }
                    } else {
                        println!("  ❌ npm login failed");
                        fail_count += 1;
                    }
                }
            }
        } else {
            let dir = root.join(target.path_relative);
            if !dir.exists() {
                 println!("    ⚠️  Directory not found: {}", dir.display());
                 fail_count += 1;
                 continue;
            }
            
            let args = if dry_run { &target.args_dry_run } else { &target.args_publish };
            
            let result = if target.is_interactive {
                run_cmd_in_dir_interactive(&dir, target.cmd, args)
            } else {
                run_cmd_in_dir(&dir, target.cmd, args)
            };
            
            match result {
                Ok(_) => {
                    println!("  ✅ {} {} successful!", target.name, if dry_run { "dry-run" } else { "publish" });
                    success_count += 1;
                }
                Err(e) => {
                    println!("  ❌ {} failed: {}", target.name, e);
                    fail_count += 1;
                }
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    if fail_count == 0 {
        println!("✅✅✅ ALL PUBLISH OPERATIONS {}! ✅✅✅", if dry_run { "VALIDATED" } else { "COMPLETE" });
    } else {
        println!("⚠️  Publish completed with {} success, {} failures", success_count, fail_count);
    }
    
    if dry_run && specific_target.is_none() {
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
// Task: test-all (Full Compliance Suite)
// =============================================================================

/// Generate the compliance test suite JSON
fn generate_test_suite() -> Result<()> {
    println!("\n🧪 Generating Compliance Test Suite...\n");
    
    run_cmd("cargo", &["run", "-p", "zakat-test-gen"])?;
    
    println!("\n✅ Test suite generated!");
    Ok(())
}

/// Run all tests across all platforms
fn run_all_tests() -> Result<()> {
    println!("\n🚀 ZakatRS Full Compliance Test Suite");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let root = project_root()?;
    let mut success_count = 0;
    let mut fail_count = 0;

    // Step 1: Generate test suite
    println!("📊 Step 1: Generating Golden Test Data...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    match run_cmd("cargo", &["run", "-p", "zakat-test-gen"]) {
        Ok(_) => {
            println!("  ✅ Test suite generated successfully!");
            success_count += 1;
        }
        Err(e) => {
            println!("  ❌ Failed to generate test suite: {}", e);
            bail!("Cannot continue without test suite.");
        }
    }

    // Step 2: Run Rust core tests
    println!("\n🦀 Step 2: Running Rust Core Tests...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    match run_cmd("cargo", &["test"]) {
        Ok(_) => {
            println!("  ✅ Rust tests passed!");
            success_count += 1;
        }
        Err(e) => {
            println!("  ❌ Rust tests failed: {}", e);
            fail_count += 1;
        }
    }

    // Step 3: Run Python compliance tests
    println!("\n🐍 Step 3: Running Python Compliance Tests...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    // Get the correct Python executable (prefer venv)
    let python = get_venv_python();
    
    // Check for maturin: first standalone, then via python -m
    let maturin_available = command_exists("maturin") || python_module_exists("maturin");
    
    if maturin_available {
        println!("  📦 Building Python wheel...");
        let build_result = if command_exists("maturin") {
            run_cmd("maturin", &["develop"])
        } else {
            run_cmd(&python, &["-m", "maturin", "develop"])
        };
        
        if build_result.is_ok() {
            // Run pytest using the venv Python
            let pytest_result = run_cmd(&python, &["-m", "pytest", "tests/py/test_compliance.py", "-v"]);
            
            match pytest_result {
                Ok(_) => {
                    println!("  ✅ Python compliance tests passed!");
                    success_count += 1;
                }
                Err(e) => {
                    println!("  ❌ Python compliance tests failed: {}", e);
                    fail_count += 1;
                }
            }
        } else {
            println!("  ⚠️ Failed to build Python wheel. Skipping Python tests.");
            fail_count += 1;
        }
    } else {
        println!("  ⚠️ maturin not found (tried 'maturin' and 'python -m maturin'). Skipping Python tests.");
    }

    // Step 4: Run Dart/Flutter compliance tests
    println!("\n💙 Step 4: Running Dart/Flutter Compliance Tests...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let dart_dir = root.join("zakat_dart");
    if dart_dir.exists() && command_exists("flutter") {
        // Get dependencies first
        println!("  📦 Getting Dart dependencies...");
        if run_cmd_in_dir(&dart_dir, "flutter", &["pub", "get"]).is_ok() {
            // Run flutter test
            match run_cmd_in_dir(&dart_dir, "flutter", &["test", "test/compliance_test.dart"]) {
                Ok(_) => {
                    println!("  ✅ Dart compliance tests passed!");
                    success_count += 1;
                }
                Err(e) => {
                    println!("  ❌ Dart compliance tests failed: {}", e);
                    fail_count += 1;
                }
            }
        } else {
            println!("  ⚠️ Failed to get Dart dependencies. Skipping Dart tests.");
            // Don't count dependency failures as test failures - they're environment issues
        }
    } else {
        if !dart_dir.exists() {
            println!("  ⚠️ zakat_dart directory not found. Skipping Dart tests.");
        } else {
            println!("  ⚠️ flutter not found. Skipping Dart tests.");
        }
    }

    // Step 5: Run Go compliance tests
    println!("\n🐹 Step 5: Running Go Compliance Tests...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let go_dir = root.join("zakat_go");
    if go_dir.exists() && command_exists("go") {
        match run_cmd_in_dir(&go_dir, "go", &["test", "-v", "./"]) {
            Ok(_) => {
                println!("  ✅ Go compliance tests passed!");
                success_count += 1;
            }
            Err(e) => {
                println!("  ❌ Go compliance tests failed: {}", e);
                fail_count += 1;
            }
        }
    } else {
        if !go_dir.exists() {
            println!("  ⚠️ zakat_go directory not found. Run 'cargo xtask build-go' first.");
        } else {
            println!("  ⚠️ go not found. Skipping Go tests.");
        }
    }

    // Step 6: Run WASM/TypeScript tests (compliance suite)
    println!("\n🕸️  Step 6: Running WASM/TypeScript Tests...");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let pkg_dir = root.join("pkg");
    if pkg_dir.exists() && command_exists("node") {
        // Copy compliance test runner to pkg directory
        let src_test = root.join("tests").join("test_wasm_compliance.js");
        let dst_test = pkg_dir.join("compliance_test.js");
        
        match copy_file(&src_test, &dst_test) {
            Ok(_) => {
                println!("  📦 Running compliance tests via Node.js...");
                // Run node compliance_test.js
                match run_cmd_in_dir(&pkg_dir, "node", &["compliance_test.js"]) {
                    Ok(_) => {
                        println!("  ✅ WASM/TypeScript tests passed!");
                        success_count += 1;
                    }
                    Err(e) => {
                        println!("  ❌ WASM/TypeScript tests failed: {}", e);
                        fail_count += 1;
                    }
                }
            },
            Err(e) => {
                 println!("  ❌ Failed to copy compliance test runner: {}", e);
                 fail_count += 1;
            }
        }
    } else {
        if !pkg_dir.exists() {
            println!("  ⚠️ pkg directory not found. Run 'cargo xtask build-all' first.");
        } else {
            println!("  ⚠️ node not found. Skipping WASM tests.");
        }
    }

    // Summary
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Test Suite Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   ✅ Passed: {}", success_count);
    println!("   ❌ Failed: {}", fail_count);
    println!();
    
    if fail_count == 0 {
        println!("✅✅✅ ALL COMPLIANCE TESTS PASSED! ✅✅✅");
        println!("\nPolyglot bindings are in sync with Rust core.");
    } else {
        println!("⚠️  Some tests failed. Please review the output above.");
        std::process::exit(1);
    }

    Ok(())
}
