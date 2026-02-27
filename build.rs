//! Build script for qcontrol Rust SDK
//!
//! Uses bindgen to generate FFI bindings from the C SDK headers,
//! ensuring the C headers remain the single source of truth for ABI types.

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    // Path to C SDK headers
    // Prefer bundled ./include/ for standalone packaging, fall back to ../c/include for in-tree dev
    let c_sdk_include = if Path::new("include/qcontrol").exists() {
        "include"
    } else {
        "../c/include"
    };

    // Tell cargo to re-run if the C headers change
    println!("cargo:rerun-if-changed={}/qcontrol/common.h", c_sdk_include);
    println!("cargo:rerun-if-changed={}/qcontrol/file.h", c_sdk_include);
    println!("cargo:rerun-if-changed=build.rs");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        // Include the main header that pulls in everything
        .header(format!("{}/qcontrol/common.h", c_sdk_include))
        .header(format!("{}/qcontrol/file.h", c_sdk_include))
        // Only generate bindings for qcontrol types
        .allowlist_type("qcontrol_.*")
        .allowlist_var("QCONTROL_.*")
        // Use core types for no_std compatibility
        .use_core()
        // Generate rustified enums where possible
        .rustified_enum("qcontrol_status_t")
        .rustified_enum("qcontrol_phase_t")
        .rustified_enum("qcontrol_error_t")
        // Don't generate layout tests (they require std)
        .layout_tests(false)
        // Parse the headers
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write bindings to OUT_DIR
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
