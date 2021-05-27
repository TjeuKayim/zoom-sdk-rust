use std::env;
use std::path::PathBuf;

fn main() {
    glue_generator::generate_glue();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_file = out_path.join("bindings.rs");
    let bundled_bindings = "bundled/bindgen.rs";
    if let Ok(_) = std::env::var("DOCS_RS") {
        // use bundled bindings because docs.rs can't run MSVC
        std::fs::copy(bundled_bindings, out_file)
            .expect("Could not copy bindings to output directory");
        return;
    }
    let env_var = "ZOOM_SDK_DIR";
    let sdk_dir = env::var(env_var).expect("Environment variable ZOOM_SDK_DIR not set");
    println!("cargo:rerun-if-env-changed={}", env_var);
    println!("cargo:rustc-link-search={}\\lib\\", sdk_dir);
    println!("cargo:rustc-link-lib=static=sdk");
    println!("cargo:rerun-if-changed=wrapper.hpp");
    println!("cargo:rerun-if-changed=glue.hpp");
    println!("cargo:rerun-if-changed=glue.cpp");
    println!("cargo:rerun-if-changed=generated.cpp");
    println!("cargo:rerun-if-changed=generated.cpp");

    cc::Build::new()
        .cpp(true)
        .include(env::current_dir().unwrap())
        .include(&format!("{}\\h", sdk_dir))
        .file("glue.cpp")
        .file(out_path.join("generated.cpp"))
        .compile("wrap.a");

    let bindings = bindgen::Builder::default()
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-v")
        .clang_arg(&format!("-I{}\\h", sdk_dir))
        .clang_arg(&format!("-I{}", out_path.to_string_lossy()))
        // TODO: Don't hard code these paths
        .clang_arg("-IC:/Program Files (x86)/Microsoft Visual Studio/2019/Enterprise/VC/Tools/MSVC/14.28.29333/include")
        .clang_arg("-IC:/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/VC/Tools/MSVC/14.27.29110/include")
        .clang_arg(r#"-IC:\Program Files (x86)\Windows Kits\10\Include\10.0.18362.0\ucrt"#)
        .clang_arg(r#"-IC:\Program Files (x86)\Windows Kits\10\Include\10.0.18362.0\shared"#)
        .clang_arg(r#"-IC:\Program Files (x86)\Windows Kits\10\Include\10.0.18362.0\um"#)
        .whitelist_function("ZOOMSDK.*")
        .whitelist_type("ZOOMSDK.*")
        .whitelist_var("ZOOMSDK.*")
        .whitelist_function("ZoomGlue.*")
        .whitelist_type("ZoomGlue.*")
        .whitelist_var("ZoomGlue.*")
        .header("wrapper.hpp")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(&out_file)
        .expect("Couldn't write bindings!");

    std::fs::copy(&out_file, bundled_bindings).unwrap();
}
