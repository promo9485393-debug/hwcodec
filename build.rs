use cc::Build;
use std::{env, path::Path};

fn main() {
    for entry in std::fs::read_dir("src").unwrap() {
        if let Ok(entry) = entry {
            println!("rerun-if-changed={}", entry.path().display());
        }
    }
    let ffi_header = "src/ffi.h";
    bindgen::builder()
        .header(ffi_header)
        .rustified_enum("*")
        .generate()
        .unwrap()
        .write_to_file(Path::new(&env::var_os("OUT_DIR").unwrap()).join("ffi.rs"))
        .unwrap();

    let mut builder = Build::new();

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-search=native=ffmpeg/windows/release/lib");
        let static_libs = [
            "avcodec", "avfilter", "avutil", "avformat", "avdevice", "mfx",
        ];
        static_libs.map(|lib| println!("cargo:rustc-link-lib=static={}", lib));
        let dyn_libs = ["User32", "bcrypt", "ole32", "advapi32"];
        dyn_libs.map(|lib| println!("cargo:rustc-link-lib={}", lib));
        builder.include("ffmpeg/windows/release/include");
    }

    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-search=native=ffmpeg/linux/release/lib");
        let static_libs = ["avcodec", "avfilter", "avutil", "avdevice", "avformat"];
        static_libs.map(|lib| println!("cargo:rustc-link-lib=static={}", lib));
        let dyn_libs = ["va", "va-drm", "va-x11", "vdpau", "X11", "z"];
        dyn_libs.map(|lib| println!("cargo:rustc-link-lib={}", lib));
        builder.include("ffmpeg/linux/release/include");
    }

    #[cfg(target_os = "macos")]
    {
        // GoDesk #243: macOS hwcodec — FFmpeg from vcpkg per-arch (arm64-osx /
        // x64-osx) since this hwcodec commit bundles only windows/linux ffmpeg.
        let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
        let triplet = if arch == "x86_64" { "x64-osx" } else { "arm64-osx" };
        let vcpkg = std::env::var("VCPKG_ROOT").expect("VCPKG_ROOT for macOS hwcodec");
        let base = format!("{}/installed/{}", vcpkg, triplet);
        println!("cargo:rustc-link-search=native={}/lib", base);
        ["avformat", "avfilter", "avdevice", "avcodec", "swscale", "swresample", "avutil"]
            .map(|lib| println!("cargo:rustc-link-lib=static={}", lib));
        ["CoreFoundation", "CoreVideo", "CoreMedia", "VideoToolbox", "AVFoundation", "AudioToolbox", "Security", "CoreServices", "CoreImage", "AppKit", "OpenGL", "Metal", "CoreGraphics", "QuartzCore"]
            .map(|fw| println!("cargo:rustc-link-lib=framework={}", fw));
        ["c++", "m", "z", "bz2", "lzma", "iconv"]
            .map(|lib| println!("cargo:rustc-link-lib={}", lib));
        builder.include(format!("{}/include", base));
    }

    builder
        .file("src/encode.c")
        .file("src/decode.c")
        .file("src/mux.c")
        .file("src/common.c")
        .file("src/data.c")
        .compile("hwcodec");
}
