use std::env;
use std::path::PathBuf;

fn main() {
    let sdk_dir = env::var("ZOOM_SDK_DIR").unwrap();
    println!("cargo:rustc-link-search={}\\lib\\", sdk_dir);
    println!("cargo:rustc-link-lib=static=sdk");
    println!("cargo:rerun-if-changed=wrapper.hpp");

    let bindings = bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-v")
        .clang_arg(&format!("-I{}\\h", sdk_dir))
        // TODO: Don't hard code these paths
        .clang_arg("-IC:/Program Files (x86)/Microsoft Visual Studio/2019/Enterprise/VC/Tools/MSVC/14.28.29333/include")
        .clang_arg("-IC:/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/VC/Tools/MSVC/14.27.29110/include")
        .clang_arg(r#"-IC:\Program Files (x86)\Windows Kits\10\Include\10.0.18362.0\ucrt"#)
        .clang_arg(r#"-IC:\Program Files (x86)\Windows Kits\10\Include\10.0.18362.0\shared"#)
        .clang_arg(r#"-IC:\Program Files (x86)\Windows Kits\10\Include\10.0.18362.0\um"#)
        .whitelist_function("ZOOMSDK.*")
        .whitelist_type("ZOOMSDK.*")
        .whitelist_var("ZOOMSDK.*")
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
