#[cfg(feature = "tonic-build")]
fn main() -> std::io::Result<()> {
    let home_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    tonic_build::configure()
        .out_dir(home_dir.join("src/pb"))
        .enum_attribute(".", "#[non_exhaustive]")
        .compile(&["./proto/autd3.proto"], &["./proto"])?;

    tonic_build::configure()
        .out_dir(home_dir.join("src/pb/lightweight"))
        .enum_attribute(".", "#[non_exhaustive]")
        .compile(&["./proto/lightweight.proto"], &["./proto"])?;

    Ok(())
}

#[cfg(not(feature = "tonic-build"))]
fn main() -> std::io::Result<()> {
    Ok(())
}
