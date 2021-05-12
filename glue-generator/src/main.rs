use clang::*;
use std::env;
use std::fs::File;
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

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut generator = GlueGenerator {
        hpp_output: File::create(out_path.join("generated.hpp")).unwrap(),
        cpp_output: File::create(out_path.join("generated.cpp")).unwrap(),
    };
    let notice = "// Programmatically generated, do not edit by hand";
    generator.write_output(notice, notice);
    generator.visit_unit(&tu);
}

struct GlueGenerator {
    hpp_output: File,
    cpp_output: File,
}

impl GlueGenerator {
    fn visit_unit(&mut self, tu: &TranslationUnit) {
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
            // if let Some(file) = namespace
            //     .get_location()
            //     .and_then(|l| l.get_file_location().file)
            // {
            //     if file.get_path().ends_with("auth_service_interface.h") {
            println!(
                "Kind {:?}, Name: {:?}, Loc: {:?}",
                &namespace.get_kind(),
                &namespace.get_display_name(),
                &namespace.get_location(),
            );
            // dbg!(namespace.get_children());
            self.visit_namespace(&namespace);
            // dbg!(&class);
        }
    }

    fn visit_namespace(&mut self, namespace: &Entity) {
        let classes = namespace
            .get_children()
            .into_iter()
            .filter(|e| e.get_kind() == EntityKind::ClassDecl);
        for class in classes {
            let children = class.get_children();
            if children.len() == 0 {
                continue;
            }
            self.visit_class(class, children)
        }
    }

    fn visit_class(&mut self, class: Entity, children: Vec<Entity>) {
        use std::fmt::Write;

        let class_name = class.get_name().unwrap();
        println!("Visit class {}", class_name);
        let mut names_seen = Vec::with_capacity(children.len());
        for member in children {
            // dbg!(&member);
            match member.get_kind() {
                EntityKind::Method => {}
                EntityKind::AccessSpecifier => continue,
                EntityKind::Destructor => continue,
                EntityKind::BaseSpecifier => continue,
                EntityKind::EnumDecl => continue,
                EntityKind::Constructor => continue,
                EntityKind::FieldDecl => continue,
                _ => panic!("Unexpected kind {:?}", &member),
            };
            if !member.is_virtual_method() {
                eprintln!("Not virtual {:?}", &member);
                continue;
            }
            let (cpp_name, glue_name) =
                rename_overloads(member.get_name().unwrap(), &mut names_seen);

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
                write!(&mut signature, ", {} {}", typ, arg.get_name().unwrap()).unwrap();
            }
            let mut definition = signature.clone();
            write!(&mut signature, ");").unwrap();
            // declaration
            let declaration = if let Some(comment) = member.get_comment() {
                let comment: String = comment
                    .split_terminator("\n")
                    .map(|l| l.trim_start())
                    .collect();
                format!("{}\n{}", comment, signature)
            } else {
                signature
            };
            // function body
            write!(&mut definition, ") {{\n    return self->{}(", cpp_name).unwrap();
            let mut arg_separator = "";
            for arg in &arguments {
                write!(
                    &mut definition,
                    "{}{}",
                    arg_separator,
                    arg.get_name().unwrap()
                )
                .unwrap();
                arg_separator = ", ";
            }
            write!(&mut definition, ");\n}}").unwrap();
            println!("{}", declaration);
            println!("{}", definition);
            self.write_output(&declaration, &mut definition);
        }
        // TODO: Generate Delete / Destructor
    }

    fn write_output(&mut self, declaration: &str, definition: &str) {
        use std::io::Write;
        writeln!(&mut self.hpp_output, "{}", &declaration).unwrap();
        writeln!(&mut self.cpp_output, "{}", &definition).unwrap();
    }
}

fn rename_overloads(name: String, names_seen: &mut Vec<Rc<String>>) -> (Rc<String>, Rc<String>) {
    use std::fmt::Write;
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
