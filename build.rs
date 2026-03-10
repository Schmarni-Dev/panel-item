use gluon_parser::parse_idl;
use gluon_codegen_rust::gen_module;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Watch for changes to KDL schema files
    let schema_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("schemas");

    // Tell cargo to rerun if any KDL files change
    println!(
        "cargo:rerun-if-changed={}",
        schema_dir.join("*.gluon").display()
    );

    // let proto =
    // Use the codegen library directly to regenerate protocol files
    let output_file =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/protocol.rs");
    let protocol = parse_idl(
        "panel_item",
        include_str!("./schemas/org.stardustxr.item.Panel.gluon"),
    )
    .unwrap();

    let proto = gen_module(&protocol);
    let str = proto.to_string();
    let str = prettyplease::unparse(&syn::parse_file(&str).unwrap());
    fs::write(output_file, str).unwrap();

    println!("Protocol files regenerated successfully");
}
