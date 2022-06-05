use std::env;
use std::process::Command;

fn main() {
    let profile = env::var("PROFILE").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    let mut bootcmp = Command::new("nasm");
    bootcmp.args(["src/boot.asm", "-felf32", "-o", &format!("{}/boot.o", out_dir)]);

   if profile == "release" {
        bootcmp.arg("-O3");
    }
    
    match bootcmp.output() {
        Ok(out) => println!("boot.asm succeeded: {}", String::from_utf8_lossy(&out.stderr)),
        Err(e) => panic!("boot.asm failed: {}", e)
    }

    println!("cargo:rustc-link-arg=-Tsrc/linker.ld");
    println!("cargo:rustc-link-arg={}/boot.o", out_dir);
}
