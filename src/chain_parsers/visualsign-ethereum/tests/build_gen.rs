// Bring in the same generator module for tests
mod build_gen {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/build_src/gen.rs"));
}

use std::fs;
use std::io::Write;

#[test]
fn normalize_selector_works() {
    assert_eq!(
        build_gen::normalize_selector("0xdeadBEEF"),
        Some("0xdeadbeef".to_string())
    );
    // keccak256("transfer(address,uint256)")[:4] = a9059cbb
    assert_eq!(
        build_gen::normalize_selector("transfer(address,uint256)"),
        Some("0xa9059cbb".to_string())
    );
}

#[test]
fn collect_entries_parses_fallback_json() {
    let tmp = tempfile::tempdir().unwrap();
    let reg_dir = tmp.path().to_path_buf();
    let json = r#"
    {
      "display": {
        "formats": {
          "0x12345678": {
            "$id": "test-format",
            "fields": [
              {"label": "Field A", "path": "data.a"},
              {"label": "Field B", "path": "data.b"}
            ]
          }
        }
      }
    }"#;
    let file_path = reg_dir.join("foo.json");
    let mut f = fs::File::create(&file_path).unwrap();
    write!(f, "{json}").unwrap();

    let entries = build_gen::collect_entries(&reg_dir);
    assert_eq!(entries.len(), 1);
    let e = &entries[0];
    assert_eq!(e.selector, "0x12345678");
    assert_eq!(e.fields.len(), 2);
    assert_eq!(e.fields[0].label, "Field A");
    assert_eq!(e.fields[0].path, "data.a");

    let generated = build_gen::generate_registry_rs(&entries);
    assert!(generated.contains("phf::phf_map!"));
    // Guard: ensure static array declarations end with semicolons
    assert!(generated.contains("static FIELDS_0: [GenField; 2] = ["));
    assert!(
        generated.contains("];\n\nstatic FORMAT_0:"),
        "FIELDS array must be closed with ]; followed by next item"
    );
    assert!(generated.contains("static FORMATS_FOR_0: [&GenFormat; 1] = ["));
    assert!(
        generated.contains("];\n\npub static SELECTOR_MAP"),
        "FORMATS_FOR array must be closed with ]; before map"
    );
}
