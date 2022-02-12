#![deny(rust_2018_idioms)]

use std::io;

mod codegen;
mod error;
mod lexer;
mod parser;
mod span;

pub use codegen::Codegen;
pub use error::{ErrorReporter, Report};
pub use lexer::Lexer;
pub use parser::{ParseError, Parser};
pub use span::{Span, Spanned};

fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("invalid arguments");
        std::process::exit(1)
    });

    let mut reporter = ErrorReporter::new(io::stderr(), &input);

    match run(&input) {
        Ok(()) => {}
        Err(e) => reporter.exit(e),
    }
}

fn run(input: &str) -> Result<(), ParseError> {
    let lexer = Lexer::new(input);
    let expr = Parser::new(lexer).expr()?;
    Codegen::new(expr, io::stdout())
        .gen()
        .expect("failed to write to stdout");

    Ok(())
}
