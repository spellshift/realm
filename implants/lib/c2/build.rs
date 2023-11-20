use std::env;
use std::path::PathBuf;
use which::which;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match env::var_os("PROTOC")
        .map(PathBuf::from)
        .or_else(|| which("protoc").ok())
    {
        Some(_) => println!("Found protoc, protos will be generated"),
        None => {
            println!("WARNING: Failed to locate protoc, protos will not be generated");
            return Ok(());
        }
    }

    match tonic_build::configure()
        .out_dir("./src")
        .build_server(false)
        .compile(&["c2.proto"], &["../../../tavern/internal/c2/"])
    {
        Err(err) => {
            println!("WARNING: Failed to compile protos: {}", err);
        }
        Ok(_) => println!("Generating protos"),
    }
    Ok(())
}
