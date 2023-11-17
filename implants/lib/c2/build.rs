fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(false)
        // .build_transport(false)
        .compile(&["c2.proto"], &["../../../tavern/internal/c2/"])?;
    // tonic_build::compile_protos("../../../tavern/internal/c2/c2.proto")?;
    Ok(())
}
