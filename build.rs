use std::fs;
use std::io::Write;
use walkdir::WalkDir;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let _ = fs::create_dir("src/proto_gen");
    let mut codegen = protoc_rust_grpc::Codegen::new();
    let mut mod_file = fs::File::create("src/proto_gen/mod.rs").unwrap();

    for entry in WalkDir::new("src/proto").into_iter().filter_map(|e| e.ok()) {
        let f_name = entry.file_name().to_string_lossy();

        if f_name.ends_with(".proto") {
            let f_len = f_name.len();
            mod_file
                .write(format!("pub mod {};\n", &f_name[..f_len - 6]).as_bytes())
                .expect("failed to write to `mod.rs`");
            codegen.input(entry.into_path());
        }
    }

    codegen
        .out_dir("src/proto_gen")
        .includes(&["src/proto"])
        .rust_protobuf(true)
        .run()
        .unwrap()
}
