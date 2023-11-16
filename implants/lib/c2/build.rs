fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../../../tavern/internal/c2/c2.proto")?;
    Ok(())
}
