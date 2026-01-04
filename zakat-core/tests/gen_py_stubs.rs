use zakat_core;

#[test]
#[cfg(all(feature = "python", feature = "stub-gen"))]
fn generate_stubs() {
    let stub = zakat_core::python::stub_info();
    stub.generate().expect("Failed to generate stubs");
    println!("Stubs generated successfully!");

    // Copy to project root for convenience
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_path = manifest_dir.join("zakatrs.pyi");
    let root_path = manifest_dir.parent().unwrap().join("zakatrs.pyi");
    
    std::fs::copy(source_path, root_path).expect("Failed to copy stubs to project root");
}
