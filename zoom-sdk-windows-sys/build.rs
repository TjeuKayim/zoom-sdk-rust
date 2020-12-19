use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search={}\\bin\\lib\\", manifest_dir);
    println!("cargo:rustc-link-lib=static=sdk");
    // println!("cargo:rustc-link-lib=user32.lib");
    // println!("cargo:rustc-link-lib=msvcrt.lib");
    // println!("cargo:rustc-link-lib=msvcprt.lib");

    println!("cargo:rerun-if-changed=wrapper.hpp");

    let bindings = bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-v")
        // .clang_arg("-m32")
        // .clang_arg("--driver-mode=cl")
        // .clang_arg("--stdlib libc++")
        .clang_arg("-I./bin/h")
        .clang_arg("-IC:/Program Files (x86)/Microsoft Visual Studio/2019/Enterprise/VC/Tools/MSVC/14.28.29333/include")
        .clang_arg(r#"-IC:\Program Files (x86)\Windows Kits\10\Include\10.0.18362.0\ucrt"#)
        .clang_arg(r#"-IC:\Program Files (x86)\Windows Kits\10\Include\10.0.18362.0\shared"#)
        .clang_arg(r#"-IC:\Program Files (x86)\Windows Kits\10\Include\10.0.18362.0\um"#)
        // .enable_cxx_namespaces()
        .opaque_type("_IMAGE_TLS_DIRECTORY64")
        // 128-bit integers don't currently have a known stable ABI
        .opaque_type("CONTEXT")
        .opaque_type("_CONTEXT")
        .opaque_type("_DISPATCHER_CONTEXT")
        .opaque_type("PCONTEXT")
        .opaque_type("PEXCEPTION_ROUTINE")
        .opaque_type("PSLIST_HEADER")
        .opaque_type("SLIST_HEADER")
        .opaque_type("_EXCEPTION_POINTERS")
        .opaque_type("LPTOP_LEVEL_EXCEPTION_FILTER")
        .opaque_type("PVECTORED_EXCEPTION_HANDLER")
        .opaque_type("LPCONTEXT")
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
