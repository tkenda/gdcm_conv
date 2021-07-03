#![allow(unused_imports)]
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "linux")]
fn build() {
    // Run GDCM cmake
    let mut cfg = cmake::Config::new("GDCM");

    let dst = cfg.define("GDCM_BUILD_TESTING", "OFF")
                 .define("GDCM_DOCUMENTATION", "OFF")
                 .define("GDCM_BUILD_EXAMPLES", "OFF")
                 .define("GDCM_BUILD_DOCBOOK_MANPAGES", "OFF")
                 .define("GDCM_SUPPORT_BROKEN_IMPLEMENTATION", "ON")
                 .cflag("-fPIC")
                 .build_arg("-j8")
                 .build();

    // Set GDCM include path
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include_dir = out_path.join("include").join("gdcm-3.1");

    // Create library
    cc::Build::new().file("convert.cc")
                    .cpp(true)
                    .cpp_link_stdlib("stdc++")
                    .flag("-fPIC")
                    .include(include_dir)
                    .warnings(false)
                    .compile("gdcm_conv");

    // Set libs paths
    println!("cargo:rustc-link-search={}", dst.join("lib").display());
    println!("cargo:rustc-link-search={}", dst.display());

    // Set libs
    println!("cargo:rustc-link-lib=static=gdcm_conv");
    println!("cargo:rustc-link-lib=stdc++");

    // GDCM libs
    println!("cargo:rustc-link-lib=static=gdcmMSFF");
    println!("cargo:rustc-link-lib=static=gdcmcharls");
    println!("cargo:rustc-link-lib=static=gdcmCommon");
    println!("cargo:rustc-link-lib=static=gdcmDICT");
    println!("cargo:rustc-link-lib=static=gdcmDSED");
    println!("cargo:rustc-link-lib=static=gdcmIOD");
    println!("cargo:rustc-link-lib=static=gdcmexpat");
    println!("cargo:rustc-link-lib=static=gdcmjpeg12");
    println!("cargo:rustc-link-lib=static=gdcmjpeg16");
    println!("cargo:rustc-link-lib=static=gdcmjpeg8");
    println!("cargo:rustc-link-lib=static=gdcmopenjp2");
    println!("cargo:rustc-link-lib=static=gdcmuuid");
    println!("cargo:rustc-link-lib=static=gdcmMEXD");
    println!("cargo:rustc-link-lib=static=gdcmzlib");
    println!("cargo:rustc-link-lib=static=socketxx");
}

#[cfg(target_os = "macos")]
fn build() {
    // Run GDCM cmake
    let mut cfg = cmake::Config::new("GDCM");

    let dst = cfg.define("GDCM_BUILD_TESTING", "OFF")
                 .define("GDCM_DOCUMENTATION", "OFF")
                 .define("GDCM_BUILD_EXAMPLES", "OFF")
                 .define("GDCM_BUILD_DOCBOOK_MANPAGES", "OFF")
                 .define("GDCM_SUPPORT_BROKEN_IMPLEMENTATION", "ON")
                 .cflag("-fPIC")
                 .build_arg("-j8")
                 .build();

    // Set GDCM include path
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include_dir = out_path.join("include").join("gdcm-3.1");

    // Create library
    cc::Build::new().file("convert.cc")
                    .cpp(true)
                    .cpp_link_stdlib("c++")
                    .flag("-fPIC")
                    .flag("-std=c++11")
                    .include(include_dir)
                    .warnings(false)
                    .compile("gdcm_conv");

    // Set libs paths
    println!("cargo:rustc-link-search={}", dst.join("lib").display());
    println!("cargo:rustc-link-search={}", dst.display());

    // Set libs
    println!("cargo:rustc-link-lib=static=gdcm_conv");
    println!("cargo:rustc-link-lib=c++");

    // GDCM libs
    println!("cargo:rustc-link-lib=static=gdcmMSFF");
    println!("cargo:rustc-link-lib=static=gdcmcharls");
    println!("cargo:rustc-link-lib=static=gdcmCommon");
    println!("cargo:rustc-link-lib=static=gdcmDICT");
    println!("cargo:rustc-link-lib=static=gdcmDSED");
    println!("cargo:rustc-link-lib=static=gdcmIOD");
    println!("cargo:rustc-link-lib=static=gdcmexpat");
    println!("cargo:rustc-link-lib=static=gdcmjpeg12");
    println!("cargo:rustc-link-lib=static=gdcmjpeg16");
    println!("cargo:rustc-link-lib=static=gdcmjpeg8");
    println!("cargo:rustc-link-lib=static=gdcmopenjp2");
    println!("cargo:rustc-link-lib=static=gdcmuuid");
    println!("cargo:rustc-link-lib=static=gdcmMEXD");
    println!("cargo:rustc-link-lib=static=gdcmzlib");
    println!("cargo:rustc-link-lib=static=socketxx");
}

#[cfg(target_os = "windows")]
fn build() {
    // Run GDCM cmake
    let mut cfg = cmake::Config::new("GDCM");

    // Configure CMAKE
    let dst = cfg.define("GDCM_BUILD_TESTING", "OFF")
                 .define("GDCM_DOCUMENTATION", "OFF")
                 .define("GDCM_BUILD_EXAMPLES", "OFF")
                 .define("GDCM_BUILD_DOCBOOK_MANPAGES", "OFF")
                 .define("GDCM_SUPPORT_BROKEN_IMPLEMENTATION", "ON")
                 .cflag("/MP8")
                 .cxxflag("/MP8")
                 .build();

    // Set GDCM include path
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let include_dir = out_path.join("include").join("gdcm-3.1");

    // Create library
    cc::Build::new().file("convert.cc")
                    .cpp(true)
                    .include(include_dir)
                    .warnings(false)
                    .compile("gdcm_conv");

    // Set libs paths
    println!("cargo:rustc-link-search={}", dst.join("lib").display());
    println!("cargo:rustc-link-search={}", dst.display());

    // Set libs
    println!("cargo:rustc-link-lib=gdcm_conv");
    println!("cargo:rustc-link-lib=rpcrt4");
    println!("cargo:rustc-link-lib=ws2_32");

    // GDCM libs
    println!("cargo:rustc-link-lib=gdcmMSFF");
    println!("cargo:rustc-link-lib=gdcmcharls");
    println!("cargo:rustc-link-lib=gdcmCommon");
    println!("cargo:rustc-link-lib=gdcmDICT");
    println!("cargo:rustc-link-lib=gdcmDSED");
    println!("cargo:rustc-link-lib=gdcmIOD");
    println!("cargo:rustc-link-lib=gdcmexpat");
    println!("cargo:rustc-link-lib=gdcmgetopt");
    println!("cargo:rustc-link-lib=gdcmjpeg12");
    println!("cargo:rustc-link-lib=gdcmjpeg16");
    println!("cargo:rustc-link-lib=gdcmjpeg8");
    println!("cargo:rustc-link-lib=gdcmopenjp2");
    println!("cargo:rustc-link-lib=gdcmMEXD");
    println!("cargo:rustc-link-lib=gdcmzlib");
    println!("cargo:rustc-link-lib=socketxx");
}

fn main() {
    // Rebuild if files change
    println!("cargo:rerun-if-changed=.");

    // Unset DESTDIR envar to avoid others libs destinations
    env::remove_var("DESTDIR");

    // Update GIT
    if !Path::new("GDCM/.git").exists() {
        let _ = Command::new("git")
            .args(&["submodule", "update", "--init"])
            .status();
    }

    build();
}