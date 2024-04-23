#[cfg(feature = "local")]
macro_rules! add {
    ($path:expr, $p:ident, $work: expr) => {
        glob::glob($path).unwrap().for_each(|entry| match entry {
            Ok($p) => {
                $work;
            }
            Err(e) => println!("{:?}", e),
        });
    };
}

#[cfg(feature = "local")]
fn main() {
    let os = if cfg!(target_os = "windows") {
        "win32"
    } else if cfg!(target_os = "macos") {
        "macosx"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        panic!("Unsupported OS");
    };

    let mut build = cc::Build::new();
    build.cpp(false);
    add!("3rdparty/SOEM/soem/*.c", path, build.file(path));
    add!(
        &format!("3rdparty/SOEM/osal/{}/*.c", os),
        path,
        build.file(path)
    );
    add!(
        &format!("3rdparty/SOEM/oshw/{}/*.c", os),
        path,
        build.file(path)
    );
    build
        .include("3rdparty/SOEM/soem")
        .include("3rdparty/SOEM/osal")
        .include(format!("3rdparty/SOEM/osal/{}", os))
        .include(format!("3rdparty/SOEM/oshw/{}", os));
    #[cfg(target_os = "windows")]
    {
        build
            .include("3rdparty/SOEM/oshw/win32/wpcap/Include")
            .include("3rdparty/SOEM/oshw/win32/wpcap/Include/pcap")
            .flag("/DWIN32");
    }
    build.compile("soem");

    #[cfg(target_os = "windows")]
    {
        let home_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

        println!("cargo:rustc-link-lib=winmm");
        println!("cargo:rustc-link-lib=ws2_32");
        if cfg!(target_arch = "arm") || cfg!(target_arch = "aarch64") {
            println!("cargo:rustc-link-search={home_dir}\\Lib\\ARM64");
        } else {
            println!("cargo:rustc-link-search={home_dir}\\Lib\\x64");
        }
        println!("cargo:rustc-link-lib=wpcap");
    }
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=pcap");
    }
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=pthread");
        println!("cargo:rustc-link-lib=rt");
    }
}

#[cfg(not(feature = "local"))]
fn main() {}
