fn main() -> std::io::Result<()> {
    tonic_build::compile_protos("./proto/autd3.proto")?;
    #[cfg(feature = "lightweight")]
    tonic_build::compile_protos("./proto/lightweight.proto")?;
    Ok(())
}
