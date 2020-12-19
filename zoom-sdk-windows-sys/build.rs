use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-search=./bin/lib");
    println!("cargo:rustc-link-lib=sdk.lib");
    println!("cargo:rustc-link-lib=user32.lib");
    println!("cargo:rustc-link-lib=msvcrt.lib");
    println!("cargo:rustc-link-lib=msvcprt.lib");

    println!("cargo:rerun-if-changed=wrapper.hpp");

    let bindings = bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-v")
        // .clang_arg("--stdlib libc++")
        .clang_arg("-I./bin/h")
        .clang_arg("-IC:/Program Files (x86)/Microsoft Visual Studio/2019/Enterprise/VC/Tools/MSVC/14.28.29333/include")
        .enable_cxx_namespaces()
        .header("wrapper.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
