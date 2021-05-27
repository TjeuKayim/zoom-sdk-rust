use clang::*;
use regex::Regex;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::rc::Rc;

pub fn generate_glue() {
    // cargo run --package glue-generator --bin glue-generator --target x86_64-pc-windows-msvc
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, true);

    let env_var = "ZOOM_SDK_DIR";
    let sdk_dir = env::var(env_var).expect("Environment variable ZOOM_SDK_DIR not set");

    let tu = index.parser("prelude.hpp")
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
    let prelude_file_name = "prelude.hpp";
    let hpp_file_name = "generated.hpp";
    let cpp_file_name = "generated.cpp";
    let mut generator = GlueGenerator {
        hpp_output: File::create(out_path.join(hpp_file_name)).unwrap(),
        cpp_output: File::create(out_path.join(cpp_file_name)).unwrap(),
    };
    let notice = "// Programmatically generated, do not edit by hand";
    generator.write_output(
        &format!(
            "{}\r\n#pragma once\r\n#include \"{}\"\r\nusing namespace ZOOMSDK;\r\n",
            notice, prelude_file_name
        ),
        &format!("{}\r\n#include \"{}\"\r\n", notice, hpp_file_name),
    );
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
        let children = namespace.get_children();
        let classes = children
            .iter()
            .filter(|e| e.get_kind() == EntityKind::ClassDecl);
        for class in classes {
            let children = class.get_children();
            if children.len() == 0 {
                continue;
            }
            self.visit_class(class, children)
        }
        // Generate New / Default for structs
        let structs = children
            .iter()
            .enumerate()
            .filter(|e| e.1.get_kind() == EntityKind::StructDecl);
        for (i, struct_e) in structs {
            if !struct_e.get_name().unwrap_or("".into()).starts_with("tag") {
                continue;
            }
            let typedef = children.get(i + 1).unwrap();
            if typedef.get_kind() != EntityKind::TypedefDecl {
                continue;
            }
            let struct_name = typedef.get_name().unwrap();
            self.generate_default_value(&struct_name, &format!("ZOOMSDK::{}", &struct_name));
        }
    }

    fn visit_class(&mut self, class: &Entity, children: Vec<Entity>) {
        use std::fmt::Write;

        let class_name = class.get_name().unwrap();
        // println!("Visit class {}", class_name);
        let mut names_seen = Vec::with_capacity(children.len());
        // Event classes have extra glue generation
        lazy_static::lazy_static! {
            static ref EVENT_INTERFACE: Regex = Regex::new("I.*Event").unwrap();
            static ref STARTS_WITH_ON: Regex = Regex::new("^(on|On).*").unwrap();
        }
        let mut event_implementation = if EVENT_INTERFACE.is_match(&class_name)
            && children
                .get(1)
                .and_then(|c| c.get_name())
                .map_or(false, |c| STARTS_WITH_ON.is_match(&c))
        {
            Some(EventImplementation::default())
        } else {
            None
        };

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

            let const_qualifier = if member.is_const_method() {
                "const "
            } else {
                ""
            };
            let result_type = member.get_result_type().unwrap().get_display_name();
            let mut signature = format!(
                "{} ZoomGlue_{}_{}({} ZOOMSDK::{1} *self",
                result_type, &class_name, &glue_name, const_qualifier,
            );
            let arguments = member.get_arguments().unwrap();
            for arg in &arguments {
                let typ = arg.get_type().unwrap().get_display_name();
                write!(&mut signature, ", {} {}", typ, arg.get_name().unwrap()).unwrap();
            }
            let mut definition = signature.clone();
            write!(&mut signature, ");").unwrap();
            // declaration
            let declaration = if let Some(comment) = member.get_comment_brief() {
                // let mut comment: String = comment
                //     .lines()
                //     .map(|l| format!("{}\r\n", l.trim_start()))
                //     .collect();
                // if !comment.ends_with("\r\n") {
                //     comment.push_str("\r\n");
                // }
                format!("/// {}\r\n{}", comment, signature)
            } else {
                signature
            };
            // function body
            write!(&mut definition, ") {{\r\n    return self->{}(", cpp_name).unwrap();
            let mut arg_separator = "";
            for arg in &arguments {
                let nam = arg.get_name().unwrap();
                write!(&mut definition, "{}{}", arg_separator, nam).unwrap();
                arg_separator = ", ";
            }
            write!(&mut definition, ");\r\n}}").unwrap();
            // println!("{}", declaration);
            // println!("{}", definition);
            self.write_output(&declaration, &mut definition);

            // Event
            if let Some(ev) = &mut event_implementation {
                let name_caps = STARTS_WITH_ON
                    .captures(&cpp_name)
                    .expect("expected event method name to start with on");
                let prefix_len = name_caps.get(1).unwrap().as_str().len();
                if result_type != "void" {
                    panic!("expected event method to return void");
                }
                let field_name = format!("cb{}", &cpp_name[prefix_len..]);
                // Field
                write!(&mut ev.fields, "  void (*{})({} *", field_name, class_name).unwrap();
                for arg in &arguments {
                    let typ = arg.get_type().unwrap().get_display_name();
                    write!(&mut ev.fields, ", {}", typ).unwrap();
                }
                write!(&mut ev.fields, ") = 0;\r\n").unwrap();
                // Method
                write!(&mut ev.methods, "  void {}(", cpp_name).unwrap();
                let mut arg_separator = "";
                for arg in &arguments {
                    let typ = arg.get_type().unwrap().get_display_name();
                    let nam = arg.get_name().unwrap();
                    write!(&mut ev.methods, "{}{} {}", arg_separator, typ, nam).unwrap();
                    arg_separator = ", ";
                }
                write!(&mut ev.methods, ") {{\r\n    if ({0}) {0}(this", field_name).unwrap();
                for arg in &arguments {
                    let nam = arg.get_name().unwrap();
                    write!(&mut ev.methods, ", {}", nam).unwrap();
                }
                write!(&mut ev.methods, ");\r\n  }}\r\n").unwrap();
            }
        }
        if let Some(ev) = &mut event_implementation {
            let impl_name = format!("ZoomGlue_{}", &class_name[1..]);
            self.write_hpp(&format!(
                "/// \\brief Generated interface implementation for callbacks.\r\nclass {}: public {} {{\r\npublic:\r\n{}{}}};",
                &impl_name, class_name, &ev.fields, &ev.methods
            ));
            self.generate_placement_new(&impl_name);
        }
        // TODO: Generate Delete / Destructor
    }

    fn generate_placement_new(&mut self, class_name: &str) {
        let signature = format!("void {0}_PlacementNew({0} *out)", class_name);
        let declaration = format!("{};", signature);
        let definition = format!("{} {{\r\n  new (out) {};\r\n}}", signature, class_name);
        self.write_output(&declaration, &definition);
    }

    fn generate_default_value(&mut self, short_name: &str, class_name: &str) {
        let signature = format!("{} ZoomGlue_{}_DefaultValue()", class_name, short_name);
        let declaration = format!("{};", signature);
        let definition = format!("{} {{\r\n  {} x; return x;\r\n}}", signature, class_name);
        self.write_output(&declaration, &definition);
    }

    fn write_output(&mut self, declaration: &str, definition: &str) {
        self.write_hpp(&declaration);
        self.write_cpp(&definition);
    }

    fn write_hpp(&mut self, code: &str) {
        use std::io::Write;
        write!(&mut self.hpp_output, "{}\r\n", &code).unwrap();
    }

    fn write_cpp(&mut self, code: &str) {
        use std::io::Write;
        write!(&mut self.cpp_output, "{}\r\n", &code).unwrap();
    }
}

fn rename_overloads(name: String, names_seen: &mut Vec<Rc<String>>) -> (Rc<String>, Rc<String>) {
    use std::fmt::Write;
    let mut name = Rc::new(name);
    names_seen.push(name.clone());
    let cpp_name = name.clone();
    let times_seen = names_seen.into_iter().filter(|n| **n == name).count() - 1;
    if times_seen > 0 {
        let mut clone = (*name).clone();
        write!(&mut clone, "{}", times_seen).unwrap();
        name = Rc::new(clone);
    }
    return (cpp_name, name);
}

#[derive(Default)]
struct EventImplementation {
    fields: String,
    methods: String,
}
