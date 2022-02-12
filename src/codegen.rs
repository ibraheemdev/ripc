use crate::parse::{Ast, BinaryExpr, BinaryOp, Expr, ExprKind, Lit};
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

    pub fn write(mut self, ast: &Ast) -> Result<(), Error> {
        self._start();
        self.start_main();

        for expr in &ast.exprs {
            self.expr(expr)?;
        }

        self.end_main();

        Ok(())
    }

    fn _start(&mut self) {
        write!(self.out, ".text\n\t").unwrap();
        write!(self.out, ".global _start\n").unwrap();

        write!(self.out, "_start:\n\t").unwrap();
        write!(self.out, "xor %ebp, %ebp\n\t").unwrap();
        write!(self.out, "call main\n\t").unwrap();
        write!(self.out, "mov $1, %eax\n\t").unwrap();
        write!(self.out, "int $0x80\n").unwrap();
    }

    fn start_main(&mut self) {
        write!(self.out, "main:\n\t").unwrap();
        write!(self.out, "push %rbp\n\t").unwrap();
        write!(self.out, "mov %rsp, %rbp\n\t").unwrap();
    }

    fn end_main(&mut self) {
        write!(self.out, "pop %rbp\n\t").unwrap();
        write!(self.out, "ret\n").unwrap();
    }

    fn expr(&mut self, expr: &Expr) -> Result<(), Error> {
        match expr.kind {
            ExprKind::Lit(WithSpan {
                value: Lit::Num(num),
                ..
            }) => write!(self.out, "mov ${}, %eax\n\t", num).unwrap(),
            ExprKind::Lit(..) => unimplemented!(),
            ExprKind::Var(i) => write!(self.out, "mov -{}(%rbp), %eax\n\t", (i + 1) * 4).unwrap(),
            ExprKind::Binary(ref expr) => self.binary_op(expr)?,
        }

        Ok(())
    }

    fn binary_op(&mut self, expr: &BinaryExpr) -> Result<(), Error> {
        if let BinaryOp::Assign = expr.op.value {
            self.expr(&expr.right)?;

            match expr.left.kind {
                ExprKind::Var(i) => {
                    write!(self.out, "mov %eax, -{}(%rbp)\n\t", (i + 1) * 4).unwrap()
                }
                _ => {
                    return Err(Error::new(ErrorKind::ExpectedIdent, expr.left.span));
                }
            }

            return Ok(());
        }

        let op = match expr.op.value {
            BinaryOp::Sub => "sub",
            BinaryOp::Add => "add",
            BinaryOp::Mul => "imul",
            BinaryOp::Div => "idiv",
            _ => return Err(Error::new(ErrorKind::InvalidOperator, expr.op.span)),
        };

        self.expr(&expr.left)?;
        write!(self.out, "push %rax\n\t").unwrap();
        self.expr(&expr.right)?;

        match expr.op.value {
            BinaryOp::Div => {
                write!(self.out, "mov %eax, %ebx\n\t").unwrap();
                write!(self.out, "pop %rax\n\t").unwrap();
                write!(self.out, "mov $0, %edx\n\t").unwrap();
                write!(self.out, "idiv %ebx\n\t").unwrap();
            }
            _ => {
                write!(self.out, "pop %rbx\n\t").unwrap();
                write!(self.out, "{} %ebx, %eax\n\t", op).unwrap();
            }
        }

        Ok(())
    }

    // fn string(&mut self, str: &str) -> Result<(), Error> {
    //     write!(self.out, "\t.data\n").unwrap();
    //     write!(self.out, ".mydata:\n\nt").unwrap();
    //     write!(self.out, ".string \"").unwrap();

    //     write!(self.out, "{}", str).unwrap();

    //     write!(self.out, "\"\n\t").unwrap();
    //     write!(self.out, ".text\n\t").unwrap();
    //     write!(self.out, ".global stringfn\n").unwrap();
    //     write!(self.out, "stringfn:\n\t").unwrap();
    //     write!(self.out, "lea .mydata(%rip), %rax\n\t").unwrap();
    //     write!(self.out, "ret\n").unwrap();

    //     Ok(())
    // }
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
    ExpectedIdent,
    InvalidOperator,
}

impl Spanned for Error {
    fn span(&self) -> Span {
        self.span
    }
}

impl<W: Write> Report<W> for Error {
    fn report(&self, f: &mut Reporter<'_, W>) -> std::io::Result<()> {
        match self.kind {
            ErrorKind::ExpectedIntExpr => write!(f.out, "Expected integer expression"),
            ErrorKind::ExpectedIdent => write!(f.out, "Expected identifier"),
            ErrorKind::InvalidOperator => write!(f.out, "Invalid operator"),
        }
    }
}
