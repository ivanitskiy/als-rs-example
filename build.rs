fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::PathBuf::from("proto");
    let proto_files = vec![root.join("envoy/service/accesslog/v2/als.proto")];
    // Tell cargo to recompile if any of these proto files are changed
    // for proto_file in &proto_files {
    //     println!("cargo:rerun-if-changed={}", proto_file.display());
    // }

    let descriptor_path =
        std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("proto_descriptor.bin");

    let mod_path = std::path::PathBuf::from("mod.rs");

    tonic_build::configure()
        .build_server(true)
        .include_file(mod_path)
        .file_descriptor_set_path(&descriptor_path)
        .compile_well_known_types(true)
        // TODO: serde(default) doesn't work on enum.
        // .type_attribute(".", "#[serde(default)]")
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(&proto_files, &["proto/"])?;

    Ok(())
}
