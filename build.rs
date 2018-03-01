extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Generate version header
    let output = Command::new("cmd")
        .args(&["/C", "UpdateGenVersion.bat"])
        .current_dir("./src/shared")
        .output()
        .expect("Failed to generate GenVersion.h");
    assert!(output.status.success());

    // Build winpty, only MSVC is supported
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
