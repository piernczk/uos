use std::collections::HashMap;
use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target = env::var("TARGET").unwrap();
    let arch = target.split('-').next().unwrap();

    let asm = HashMap::from([
        ("i686", ["boot", "int"])
    ]);

    for file in asm[arch] {
        let mut cmd = Command::new("nasm");
        cmd.args([&format!("src/arch/{}/{}.asm", arch, file), "-felf32", "-O3", "-o", &format!("{}/{}.o", out_dir, file)]);

        match cmd.output() {
            Ok(out) => println!("{}.asm succeeded: {}", file, String::from_utf8_lossy(&out.stderr)),
            Err(e) => panic!("{}.asm failed: {}", file, e)
        }

        println!("cargo:rerun-if-changed=src/arch/{}/{}.asm", arch, file);
        println!("cargo:rustc-link-arg={}/{}.o", out_dir, file);
    }

    println!("cargo:rerun-if-changed=src/arch/{}/linker.ld", arch);
    println!("cargo:rustc-link-arg=-Tukern/src/arch/{}/linker.ld", arch);
}
