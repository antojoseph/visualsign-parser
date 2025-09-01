use std::{env, fs, io::Write, path::PathBuf};

mod build_gen {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/build_src/gen.rs"));
}

fn main() {
    // Directory containing the JSON registry specs
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let registry_dir = manifest_dir.join("static/eip7730/registry");
    println!("cargo:rerun-if-changed={}", registry_dir.display());
    // Also rerun if the generator itself changes
    let gen_src = manifest_dir.join("build_src/gen.rs");
    println!("cargo:rerun-if-changed={}", gen_src.display());

    // Collect entries and generate Rust code
    let entries = build_gen::collect_entries(&registry_dir);
    let generated = build_gen::generate_registry_rs(&entries);

    // Write to OUT_DIR
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest_path = out_dir.join("erc7730_registry_gen.rs");
    let mut file = fs::File::create(&dest_path).unwrap();
    writeln!(file, "{generated}").unwrap();
}
