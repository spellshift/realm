fn main() -> Result<(), Box<dyn std::error::Error>> {
    match tonic_build::configure()
        .out_dir("./src")
        .build_server(false)
        .compile(&["c2.proto"], &["../../../tavern/internal/c2/"])
    {
        Err(err) => println!("WARNING: Failed to compile protos: {}", err),
        Ok(_) => println!("Generating protos"),
    }
    Ok(())
}
