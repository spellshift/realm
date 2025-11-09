use std::env;
use std::path::PathBuf;
use which::which;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Skip if no `protoc` can be found
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

    // Build Eldritch Proto
    match tonic_build::configure()
        .out_dir("./src/generated/")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_client(false)
        .build_server(false)
        .compile(&["eldritch.proto"], &["../../../tavern/internal/c2/proto"])
    {
        Err(err) => {
            println!("WARNING: Failed to compile eldritch protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated eldritch protos"),
    };

    // Build C2 Protos
    match tonic_build::configure()
        .out_dir("./src/generated")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_server(false)
        .extern_path(".eldritch", "crate::eldritch")
        .compile(&["c2.proto"], &["../../../tavern/internal/c2/proto/"])
    {
        Err(err) => {
            println!("WARNING: Failed to compile c2 protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated c2 protos"),
    };

    Ok(())
}
