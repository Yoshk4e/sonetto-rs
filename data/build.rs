use std::env;
use std::fs;
use std::path::Path;

mod codegen;

//add tables to codegen/tables

fn main() {
    println!("cargo:rerun-if-changed=excel2json");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=codegen");

    let json_dir = if let Ok(custom_dir) = env::var("JSON_DATA_DIR") {
        custom_dir
    } else {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        Path::new(&manifest_dir)
            .join("excel2json")
            .to_string_lossy()
            .to_string()
    };

    let output_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("src")
        .join("exceldb");

    fs::create_dir_all(&output_dir).unwrap();

    if let Err(e) = codegen::generate_rust_modules(&json_dir, &output_dir.to_string_lossy()) {
        eprintln!("Failed to generate Rust modules: {}", e);
        std::process::exit(1);
    }

    println!("Generated Rust modules in {:?}", output_dir);
}
