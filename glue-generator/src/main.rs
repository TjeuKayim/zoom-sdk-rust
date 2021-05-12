use clang::*;
use std::env;
use std::fmt::Write;
use std::path::PathBuf;
use std::rc::Rc;

fn main() {
    // cargo run --package glue-generator --bin glue-generator --target x86_64-pc-windows-msvc
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, true);

    let env_var = "ZOOM_SDK_DIR";
    let sdk_dir = env::var(env_var).expect("Environment variable ZOOM_SDK_DIR not set");
    let mut header_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    header_path.push(r#"..\zoom-sdk-windows-sys\prelude.hpp"#);
    dbg!(&header_path);
    let header_path = header_path.canonicalize().unwrap();
    dbg!(&header_path);

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
        .filter(|e| {
            e.get_kind() == EntityKind::Namespace
                && e.get_name().map(|n| &n == "ZOOMSDK").unwrap_or(false)
        })
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
        let children = class.get_children();
        if children.len() == 0 {
            continue;
        }
        visit_class(class, children)
    }
}

fn visit_class(class: Entity, children: Vec<Entity>) {
    let class_name = class.get_name().unwrap();
    println!("Visit class {}", class_name);
    let mut names_seen = Vec::with_capacity(children.len());
    // dbg!(class.get_comment());
    // let comment = class.get_comment_brief().unwrap();
    for member in children {
        // dbg!(&member);
        match member.get_kind() {
            EntityKind::Method => {}
            EntityKind::AccessSpecifier => continue,
            EntityKind::Destructor => continue,
            _ => panic!("Unexpected kind {:?}", &member),
        };
        if !member.is_virtual_method() {
            eprintln!("Not virtual {:?}", &member);
            continue;
        }
        let (cpp_name, glue_name) = rename(member.get_name().unwrap(), &mut names_seen);

        // dbg!(member.get_arguments());
        // dbg!(member.is_const_method());
        // dbg!(member.get_result_type());

        let mut signature = format!(
            "{}{} ZoomSdkGlue_{}_{}({2} *self",
            if member.is_const_method() {
                "const "
            } else {
                ""
            },
            member.get_result_type().unwrap().get_display_name(),
            &class_name,
            &glue_name,
        );
        let arguments = member.get_arguments().unwrap();
        for arg in &arguments {
            let typ = arg.get_type().unwrap().get_display_name();
            write!(&mut signature, ", {} {}", typ, arg.get_name().unwrap());
        }
        let mut definition = signature.clone();
        write!(&mut signature, ");");
        let declaration = signature;
        // function body
        write!(&mut definition, ") {{\n    return self->{}(", cpp_name);
        let mut arg_separator = "";
        for arg in &arguments {
            write!(
                &mut definition,
                "{}{}",
                arg_separator,
                arg.get_name().unwrap()
            );
            arg_separator = ", ";
        }
        write!(&mut definition, ");\n}}");
        println!("{}", declaration);
        println!("{}", definition);
    }
}

// struct NameMap {
//     rust: String,
//     cpp: String,
// }

fn visit_class_member(class_name: &str, member: &Entity) {}

fn rename(name: String, names_seen: &mut Vec<Rc<String>>) -> (Rc<String>, Rc<String>) {
    let mut name = Rc::new(name);
    names_seen.push(name.clone());
    let cpp_name = name.clone();
    let times_seen = names_seen.into_iter().filter(|n| **n == name).count();
    if times_seen > 0 {
        let mut clone = (*name).clone();
        write!(&mut clone, "{}", times_seen).unwrap();
        name = Rc::new(clone);
    }
    return (cpp_name, name);
}
