use crate::parser::Expr;

use std::io::{self, Write};

pub struct Codegen<W> {
    out: W,
    expr: Expr,
}

impl<W> Codegen<W>
where
    W: Write,
{
    pub fn new(expr: Expr, out: W) -> Self {
        Self { expr, out }
    }

    pub fn gen(mut self) -> Result<(), io::Error> {
        write!(self.out, "  .globl main")?;
        write!(self.out, "main:")?;
        // ...
        write!(self.out, "  ret")
    }
}
