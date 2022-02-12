use crate::parse::{BinaryExpr, BinaryOp, Expr, ExprKind, Lit};
use crate::{Report, Reporter, Span, Spanned, WithSpan};

use std::io::Write;

pub struct Codegen<W> {
    out: W,
}

impl<W> Codegen<W>
where
    W: Write,
{
    pub fn new(out: W) -> Self {
        Self { out }
    }

    pub fn write(mut self, expr: &Expr) -> Result<(), Error> {
        match &expr.kind {
            ExprKind::Lit(WithSpan {
                value: Lit::String(ref str),
                ..
            }) => {
                self.emit_string(str)?;
            }
            _ => {
                self.emit_start();
                self.emit_main(expr)?;
            }
        }

        Ok(())
    }

    fn emit_start(&mut self) {
        write!(self.out, ".text\n\t").unwrap();
        write!(self.out, ".global _start\n").unwrap();

        write!(self.out, "_start:\n\t").unwrap();
        write!(self.out, "call main\n\t").unwrap();
        write!(self.out, "mov $1, %eax\n\t").unwrap();
        write!(self.out, "int $0x80\n").unwrap();
    }

    fn emit_main(&mut self, expr: &Expr) -> Result<(), Error> {
        write!(self.out, "main:\n\t").unwrap();
        self.emit_int_expr(expr)?;
        write!(self.out, "ret\n").unwrap();

        Ok(())
    }

    fn emit_string(&mut self, str: &str) -> Result<(), Error> {
        write!(self.out, "\t.data\n").unwrap();
        write!(self.out, ".mydata:\n\nt").unwrap();
        write!(self.out, ".string \"").unwrap();

        write!(self.out, "{}", str).unwrap();

        write!(self.out, "\"\n\t").unwrap();
        write!(self.out, ".text\n\t").unwrap();
        write!(self.out, ".global stringfn\n").unwrap();
        write!(self.out, "stringfn:\n\t").unwrap();
        write!(self.out, "lea .mydata(%rip), %rax\n\t").unwrap();
        write!(self.out, "ret\n").unwrap();

        Ok(())
    }

    fn emit_int_expr(&mut self, expr: &Expr) -> Result<(), Error> {
        match &expr.kind {
            ExprKind::Lit(WithSpan {
                value: Lit::Num(num),
                ..
            }) => {
                write!(self.out, "mov ${}, %eax\n\t", num).unwrap();
            }
            ExprKind::Binary(expr) => {
                self.emit_binary(expr)?;
            }
            _ => return Err(Error::new(ErrorKind::ExpectedIntExpr, expr.span)),
        }

        Ok(())
    }

    fn emit_binary(&mut self, expr: &BinaryExpr) -> Result<(), Error> {
        self.emit_int_expr(&expr.left)?;
        write!(self.out, "push %rax\n\t").unwrap();
        self.emit_int_expr(&expr.right)?;

        match expr.op.value {
            BinaryOp::Div => {
                write!(self.out, "mov %eax, %ebx\n\t").unwrap();
                write!(self.out, "pop %rax\n\t").unwrap();
                write!(self.out, "mov $0, %edx\n\t").unwrap();
                write!(self.out, "idiv %ebx\n\t").unwrap();
            }
            ref op => {
                let op = match op {
                    BinaryOp::Sub => "sub",
                    BinaryOp::Add => "add",
                    BinaryOp::Mul => "imul",
                    BinaryOp::Div => unreachable!(),
                };

                write!(self.out, "pop %rax\n\t").unwrap();
                write!(self.out, "{} %ebx, %eax\n\t", op).unwrap();
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
}

impl Error {
    fn new(kind: ErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ErrorKind {
    ExpectedIntExpr,
}

impl Spanned for Error {
    fn span(&self) -> Span {
        self.span
    }
}

impl<W: Write> Report<W> for Error {
    fn report(&self, f: &mut Reporter<'_, W>) -> std::io::Result<()> {
        match self.kind {
            ErrorKind::ExpectedIntExpr => write!(f.out, "expected integer expression"),
        }
    }
}
