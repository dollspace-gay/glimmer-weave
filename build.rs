/**
 * Build script for Glimmer-Weave
 *
 * This script links the native allocator assembly code with the Rust tests.
 * It runs automatically when you run `cargo build` or `cargo test`.
 *
 * NOTE: The native allocator uses AT&T syntax assembly (.S files) which requires
 * GNU assembler (gas). This is available on:
 * - Linux (via gcc)
 * - macOS (via clang)
 * - Windows with MinGW-w64 or MSYS2
 *
 * On Windows with MSVC, the allocator tests will be skipped.
 */

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=src/native_allocator.S");

    let target = env::var("TARGET").unwrap();

    // Check if we're on a platform that supports GNU assembler
    let has_gas = !target.contains("msvc");

    if target.contains("x86_64") && has_gas {
        // Try to compile the assembly file
        println!("cargo:warning=Compiling native allocator with GNU assembler");

        match cc::Build::new()
            .file("src/native_allocator.S")
            .try_compile("native_allocator")
        {
            Ok(_) => {
                // Successfully compiled - enable the allocator_tests feature
                println!("cargo:rustc-cfg=feature=\"allocator_tests\"");
                println!("cargo:warning=Native allocator compiled successfully");

                // Tell cargo to link the compiled library
                let out_dir = env::var("OUT_DIR").unwrap();
                println!("cargo:rustc-link-search=native={}", out_dir);
                println!("cargo:rustc-link-lib=static=native_allocator");
            }
            Err(e) => {
                println!("cargo:warning=Failed to compile native allocator: {}", e);
                println!("cargo:warning=Allocator tests will be skipped");
                println!("cargo:warning=Install gcc/gas to enable allocator tests");
            }
        }
    } else if target.contains("msvc") {
        println!("cargo:warning=Native allocator requires GNU assembler (not available with MSVC)");
        println!("cargo:warning=Allocator tests will be skipped");
        println!("cargo:warning=To run allocator tests on Windows:");
        println!("cargo:warning=  1. Install MSYS2 from https://www.msys2.org/");
        println!("cargo:warning=  2. Install MinGW toolchain: pacman -S mingw-w64-x86_64-toolchain");
        println!("cargo:warning=  3. Use target: cargo test --target x86_64-pc-windows-gnu");
        println!("cargo:warning=Or run tests on Linux/macOS where GNU assembler is available");
    } else {
        println!("cargo:warning=Native allocator only supports x86_64 with GNU assembler");
        println!("cargo:warning=Current target: {}", target);
        println!("cargo:warning=Allocator tests will be skipped");
    }
}
