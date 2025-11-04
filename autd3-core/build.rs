#[cfg(not(target_os = "windows"))]
fn main() {}

#[cfg(target_os = "windows")]
fn main() {
    #[cfg(feature = "sleep")]
    println!("cargo:rustc-link-lib=winmm");
}
