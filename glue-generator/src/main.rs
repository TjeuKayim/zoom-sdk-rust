use clang::*;
use std::env;
use std::path::PathBuf;

fn main() {
    // cargo run --package glue-generator --bin glue-generator --target x86_64-pc-windows-msvc
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, true);

    let env_var = "ZOOM_SDK_DIR";
    let sdk_dir = env::var(env_var).expect("Environment variable ZOOM_SDK_DIR not set");
    // let mut header_path = PathBuf::from(sdk_dir);
    // header_path.push("h/auth_service_interface.h");
    let header_path = r#"C:\Users\tjeuk\src\zoom-sdk-rust\zoom-sdk-windows-sys\wrapper.hpp"#;

    let tu = index.parser(&header_path)
        .arguments(&[
            "-x",
            "c++",
            "-v",
            &format!("-I{}\\h", sdk_dir),
            "-IC:/Program Files (x86)/Microsoft Visual Studio/2019/Enterprise/VC/Tools/MSVC/14.28.29333/include",
            "-IC:/Program Files (x86)/Microsoft Visual Studio/2019/BuildTools/VC/Tools/MSVC/14.27.29110/include",
            r#"-IC:/Program Files (x86)/Windows Kits/10/Include/10.0.18362.0/ucrt"#,
            r#"-IC:/Program Files (x86)/Windows Kits/10/Include/10.0.18362.0/shared"#,
            r#"-IC:/Program Files (x86)/Windows Kits/10/Include/10.0.18362.0/um"#,
        ])
        .parse().unwrap();

    let namespaces = tu
        .get_entity()
        .get_children()
        .into_iter()
        .filter(|e| e.get_kind() == EntityKind::Namespace)
        .collect::<Vec<_>>();

    for namespace in namespaces {
        if let Some(file) = namespace
            .get_location()
            .and_then(|l| l.get_file_location().file)
        {
            if file.get_path().ends_with("auth_service_interface.h") {
                println!(
                    "Kind {:?}, Name: {:?}, Loc: {:?}",
                    &namespace.get_kind(),
                    &namespace.get_display_name(),
                    &namespace.get_location(),
                );
                // dbg!(namespace.get_children());
                visit_namespace(&namespace);
            }
        }
        // dbg!(&class);
    }
}

fn visit_namespace(namespace: &Entity) {
    let classes = namespace
        .get_children()
        .into_iter()
        .filter(|e| e.get_kind() == EntityKind::ClassDecl);
    for class in classes {
        let name = class.get_name().unwrap();
        // dbg!(class.get_name());
        // dbg!(class.get_mangled_name());
        // dbg!(class.get_comment());
        // dbg!(class.get_comment_brief());
        // dbg!(class.get_parsed_comment());
        // dbg!(class.get_comment_range());
        let comment = class.get_comment_brief().unwrap();
    }
}

fn visit_method(method: &Entity) {
    dbg!(method.get_arguments());
    dbg!(method.is_const_method());
    dbg!(method.is_virtual_method());
    // dbg!(method.dest());
    // TODO: is destructor
    dbg!(method.get_result_type());
}
