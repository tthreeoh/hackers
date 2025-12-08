use hackers::hackrs::HaCKS::HaCKS;
use std::path::PathBuf;

#[test]
fn test_dynamic_loading() {
    let mut hacks = HaCKS::new();

    // Locate the DLL
    let mut dll_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dll_path.push("examples");
    dll_path.push("example_dll");
    dll_path.push("target");
    dll_path.push("debug");
    dll_path.push("example_dll.dll");

    println!("Loading DLL from: {:?}", dll_path);
    assert!(dll_path.exists(), "DLL not found at {:?}. Did you run 'cargo build --manifest-path examples/example_dll/Cargo.toml'?", dll_path);

    hacks
        .load_dynamic(&dll_path)
        .expect("Failed to load dynamic module");

    // Verify it's loaded in the registry
    assert!(
        !hacks.hacs.is_empty(),
        "hacs should not be empty after loading dynamic module"
    );

    // Verify registry
    // The module registers itself.
    // We can't easily check for ForeignHaCK specifically without its TypeId,
    // but we can check if count of modules > 0.
    // However, HaCKS starts empty? Yes.
    assert!(!hacks.hacs.is_empty(), "hacs registry should not be empty");
}
