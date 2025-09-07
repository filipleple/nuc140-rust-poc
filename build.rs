use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // 1) Ensure the shim exists (lets us call SYS_UnlockReg / SYS_LockReg from Rust)
    let shim = "bsp/bsp_init.c";
    if !Path::new(shim).exists() {
        fs::write(
            shim,
            br#"#include "SYS.h"
void SYS_UnlockReg(void){ UNLOCKREG(); }
void SYS_LockReg(void){ LOCKREG(); }
"#,
        )
        .unwrap();
        println!("cargo:rerun-if-changed={}", shim);
    }

    // 2) Build the C BSP into a static library
    let mut b = cc::Build::new();
    b.target("thumbv6m-none-eabi")
        .compiler("arm-none-eabi-gcc")
        .flag("-mcpu=cortex-m0")
        .flag("-mthumb")
        .flag("-ffunction-sections")
        .flag("-fdata-sections")
        .flag("-fno-builtin")
        .flag("-ffreestanding")
        .flag("-Os")
        .flag("-std=gnu99")
        // Your Makefile defines:
        .define("__EVAL", None)
        .define("__UVISION_VERSION", Some("542"))
        // Include paths (from your Makefile):
        .include("Library/ff8/src") // harmless even if unused
        .include("Library/CMSIS/Include")
        .include("Library/Device/Nuvoton/NUC1xx/Include")
        .include("Library/Device/Nuvoton/NUC1xx/Source")
        .include("Library/NUC1xx/Include")
        .include("Library/NUC1xx-LB_002/Include")
        // C sources we actually need for "Hello LCD":
        .file("Library/NUC1xx/Source/SYS.c")
        .file("Library/NUC1xx/Source/SPI.c")
        .file("Library/NUC1xx/Source/GPIO.c")
        .file("Library/NUC1xx-LB_002/Source/LCD.c")
        .file("Library/Device/Nuvoton/NUC1xx/Source/system_NUC1xx.c")
        .file("bsp/bsp_init.c"); // the shim

    // IMPORTANT: DO NOT add the vendor startup_gcc.S (Rust provides startup)
    b.compile("bsp");

    // Tell rust-lld where libgcc is and to link it.
    // This works across GCC versions/paths.
    let gcc = b.get_compiler();
    let out = std::process::Command::new(gcc.path())
        .arg("-print-libgcc-file-name")
        .output()
        .expect("failed to run -print-libgcc-file-name");
    let libgcc_path = String::from_utf8(out.stdout).unwrap();
    let libgcc_path = libgcc_path.trim();
    println!("cargo:rustc-link-arg={}", libgcc_path);

    // (Optional) group with your BSP to be extra safe on archive ordering
    println!("cargo:rustc-link-arg=--start-group");
    println!("cargo:rustc-link-lib=static=bsp");
    println!("cargo:rustc-link-arg=--end-group");
    

    // Re-run if these change
    println!("cargo:rerun-if-changed=Library/NUC1xx/Source/SYS.c");
    println!("cargo:rerun-if-changed=Library/NUC1xx/Source/GPIO.c");
    println!("cargo:rerun-if-changed=Library/NUC1xx-LB_002/Source/LCD.c");
    println!("cargo:rerun-if-changed=Library/Device/Nuvoton/NUC1xx/Source/system_NUC1xx.c");
    println!("cargo:rerun-if-changed=nuc140.ld");
    println!("cargo:rerun-if-changed=src/main.rs");
}
