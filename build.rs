extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!(
        "cargo:rustc-link-search=native=C:/Program Files (x86)/National Instruments/Shared/ExternalCompilerSupport/C/lib64/msvc/",
    );
    println!(
        "cargo:rustc-link-search=native=C:/Program Files/National Instruments/Shared/ExternalCompilerSupport/C/lib64/msvc/",
    );
    println!("cargo:rustc-link-lib=NIDAQmx");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++17")
        .clang_arg("-IC:/Program Files (x86)/National Instruments/Shared/ExternalCompilerSupport/C/include")
        .clang_arg("-IC:/Program Files/National Instruments/Shared/ExternalCompilerSupport/C/include")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .default_macro_constant_type(bindgen::MacroTypeVariation::Signed)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
