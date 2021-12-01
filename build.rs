fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from("mod.rs");
    tonic_build::configure()
        .build_server(true)
        .include_file(path)
        .compile(
            &["proto/envoy/service/accesslog/v2/als.proto"],
            &["proto/"],
        )?;
    Ok(())
}
