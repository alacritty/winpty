extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;
use std::process::Command;
#[cfg(feature="winpty-agent")]
use std::fs::copy;
use std::str::from_utf8;

fn main() {
    // Generate version header
    let output = Command::new("cmd")
        .args(&["/C", "UpdateGenVersion.bat"])
        .current_dir("./src/shared")
        .output()
        .expect("Failed to generate GenVersion.h");
    if !output.status.success() {
        panic!("{}", from_utf8(&output.stdout).unwrap());
    }

    #[cfg(feature = "winpty-agent")]
    {
        let arch = if cfg!(target_arch = "x86_64") {
            "x64"
        } else {
            "Win32"
        };

        // Generate solution file for MSVC
        if !Path::new("src/winpty.sln").exists() {
            let output = Command::new("gyp")
                .args(&[
                    "-I", "configurations.gypi"
                ])
                .current_dir("src")
                .output()
                .expect("Failed to generate winpty.sln. Is gyp in your path?");

            if !output.status.success() {
                panic!("{}", from_utf8(&output.stdout).unwrap());
            }
        }

        // Build winpty-agent binary
        let output = Command::new("msbuild")
            .args(&[
                "src/winpty.sln",
                "/target:winpty-agent",
                "/p:Configuration=Release",
            ])
            .arg(format!("/p:Platform={}", arch))
            .output()
            .expect("Failed to build winpty-agent. Is msbuild in your path?");

        if !output.status.success() {
            panic!("{}", from_utf8(&output.stdout).unwrap());
        }

        // Copy generated winpty-agent to rust target directory
        copy(
            format!("src/Release/{}/winpty-agent.exe", arch),
            PathBuf::from(env::var("OUT_DIR").unwrap()).join("winpty-agent.exe"),
        ).unwrap();
    }

    // Build winpty library, only MSVC is supported
    // 32bit *should* work but hasn't been tested
    cc::Build::new()
        .cpp(true)
        .include("src/libwinpty")
        .include("src/include")
        .include("src/shared")
        .include("src/gen")
        .file("src/libwinpty/AgentLocation.cc")
        .file("src/libwinpty/winpty.cc")
        .file("src/shared/BackgroundDesktop.cc")
        .file("src/shared/Buffer.cc")
        .file("src/shared/DebugClient.cc")
        .file("src/shared/GenRandom.cc")
        .file("src/shared/OwnedHandle.cc")
        .file("src/shared/StringUtil.cc")
        .file("src/shared/WindowsSecurity.cc")
        .file("src/shared/WindowsVersion.cc")
        .file("src/shared/WinptyAssert.cc")
        .file("src/shared/WinptyException.cc")
        .file("src/shared/WinptyVersion.cc")
        .define("COMPILING_WINPTY_DLL", None)
        .flag("/EHsc") // Exception handling
        .compile("winpty");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("src/include/winpty.h")
        .clang_arg("-x")
        .clang_arg("c++")
        // This breaks the bindings at the moment
        .rustfmt_bindings(false)
        .blacklist_type("_IMAGE_LINENUMBER")
        .blacklist_type("_IMAGE_LINENUMBER__bindgen_ty_1")
        .blacklist_type("IMAGE_LINENUMBER")
        .blacklist_type("PIMAGE_LINENUMBER")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=build.rs");
}
