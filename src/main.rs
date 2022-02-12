#![deny(rust_2018_idioms)]

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::io;
use std::io::Write;

mod codegen;
mod error;
mod lex;
mod parse;
mod span;

pub use codegen::Codegen;
pub use error::{Report, Reporter};
pub use lex::Lexer;
pub use parse::Parser;
pub use span::{Span, Spanned, WithSpan};

fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("invalid arguments");
        std::process::exit(1)
    });

    let mut reporter = Reporter::new(io::stderr(), &input);

    match run(&input) {
        Ok(()) => {}
        Err(e) => reporter.exit(e),
    }
}

fn run(input: &str) -> Result<(), Box<dyn Report<io::Stderr>>> {
    let lexer = Lexer::new(input);
    let expr = Parser::new(lexer).expr()?;

    let mut out = Vec::new();
    Codegen::new(&mut out).emit(&expr)?;

    match std::fs::create_dir("./ripc-target") {
        Err(err) if err.kind() != io::ErrorKind::AlreadyExists => {
            panic!("failed to create target directory: {}", err)
        }
        _ => {}
    };

    let hash = {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(fastrand::u64(..));
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
        .arg("-o")
        .arg(&out_file)
        .status()
        .expect("failed to assemble output");

    std::process::Command::new("ld")
        .arg(&out_file)
        .arg("-o")
        .arg("out")
        .status()
        .expect("linking failed");

    Ok(())
}
