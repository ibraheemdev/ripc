use crate::codegen::{self, Codegen};
use crate::parse::Ast;
use crate::rand;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io::{self, Write};

pub fn emit(ast: &Ast) -> Result<(), codegen::Error> {
    let mut out = Vec::new();
    Codegen::new(&mut out).write(&ast)?;

    match std::fs::create_dir("./ripc-target") {
        Err(err) if err.kind() != io::ErrorKind::AlreadyExists => {
            panic!("failed to create target directory: {}", err)
        }
        _ => {}
    };

    let hash = {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(rand::rand());
        hasher.finish()
    };

    let asm_file = format!("./ripc-target/{}.s", hash);
    let out_file = format!("./ripc-target/{}.o", hash);

    std::fs::File::create(&asm_file)
        .expect("failed to open output file")
        .write_all(&out)
        .expect("failed to write output");

    std::process::Command::new("as")
        .arg(&asm_file)
        .arg("-g")
        .arg("-o")
        .arg(&out_file)
        .status()
        .expect("failed to assemble output");

    std::process::Command::new("ld")
        .arg("-o")
        .arg("out")
        .arg("--dynamic-linker")
        .arg("/lib64/ld-linux-x86-64.so.2")
        .arg(&out_file)
        .arg("-lc")
        .status()
        .expect("linking failed");

    Ok(())
}
