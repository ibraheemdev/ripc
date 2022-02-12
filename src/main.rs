#![deny(rust_2018_idioms)]

use std::io;

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
    Codegen::new(io::stdout()).emit(&expr)?;

    Ok(())
}
