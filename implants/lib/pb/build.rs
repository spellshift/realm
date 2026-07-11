use std::env;
use std::path::PathBuf;
use which::which;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only the truly build-system concerns stay here.
    println!("cargo:rerun-if-env-changed=PROTOC");
    println!("cargo:rerun-if-env-changed=IMIX_DEBUG");
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let imix_debug = std::env::var("IMIX_DEBUG").unwrap_or_default();

    if profile == "debug" || imix_debug == "tomes" || imix_debug == "all" {
        println!("cargo:rustc-cfg=feature=\"print_debug_tome\"");
    }

    if profile == "debug" || imix_debug == "all" {
        println!("cargo:rustc-cfg=feature=\"print_debug\"");
    }

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

    match tonic_prost_build::configure()
        .out_dir("./src/generated/")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_client(false)
        .build_server(false)
        .compile_protos(
            &["eldritch.proto"],
            &[
                "../../../tavern/internal/c2/proto/",
                "../../../tavern/portals/proto/",
            ],
        ) {
        Err(err) => {
            println!("WARNING: Failed to compile eldritch protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated eldritch protos"),
    };

    match tonic_prost_build::configure()
        .out_dir("./src/generated/")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_client(false)
        .build_server(false)
        .compile_protos(
            &["portal.proto"],
            &[
                "../../../tavern/internal/c2/proto/",
                "../../../tavern/portals/proto/",
            ],
        ) {
        Err(err) => {
            println!("WARNING: Failed to compile portal protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated portal protos"),
    };

    match tonic_prost_build::configure()
        .out_dir("./src/generated/")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_client(false)
        .build_server(false)
        .compile_protos(&["trace.proto"], &["../../../tavern/portals/proto/"])
    {
        Err(err) => {
            println!("WARNING: Failed to compile portal protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated portal trace protos"),
    };

    match tonic_prost_build::configure()
        .out_dir("./src/generated")
        .codec_path("crate::xchacha::ChachaCodec")
        .build_server(false)
        .extern_path(".eldritch", "crate::eldritch")
        .compile_protos(
            &["c2.proto"],
            &[
                "../../../tavern/internal/c2/proto/",
                "../../../tavern/portals/proto/",
            ],
        ) {
        Err(err) => {
            println!("WARNING: Failed to compile c2 protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated c2 protos"),
    };

    match tonic_prost_build::configure()
        .out_dir("./src/generated")
        .build_server(false)
        .build_client(false)
        .compile_protos(
            &["conv.proto"],
            &[
                "../../../tavern/internal/c2/proto/",
                "../../../tavern/portals/proto/",
            ],
        ) {
        Err(err) => {
            println!("WARNING: Failed to compile conv protos: {}", err);
            panic!("{}", err);
        }
        Ok(_) => println!("generated conv protos"),
    };

    Ok(())
}
