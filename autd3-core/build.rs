#[cfg(not(target_os = "windows"))]
fn main() {}

#[cfg(target_os = "windows")]
fn main() {
    println!("cargo:rustc-link-lib=winmm");
}
