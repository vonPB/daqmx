extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!(
        "cargo:rustc-link-search=native={}/externals/lib64/msvc/",
        dir
    );

    println!("cargo:rustc-link-lib=NIDAQmx");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++17")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
