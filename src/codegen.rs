use crate::parse::{Ast, BinaryExpr, BinaryOp, Call, Expr, ExprKind, Lit};
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
        self.entry();
        self.start_main();

        for expr in &ast.exprs {
            self.expr(expr)?;
        }

        self.end_main();

        Ok(())
    }

    fn entry(&mut self) {
        asm!(self, ".text\n\t");
        asm!(self, ".global _start\n");

        asm!(self, "_start:\n\t");
        asm!(self, "xor %ebp, %ebp\n\t");
        asm!(self, "call main\n\t");
        asm!(self, "mov $1, %edi\n\t");
        asm!(self, "call exit\n");
    }

    fn start_main(&mut self) {
        asm!(self, "main:\n\t");
        asm!(self, "push %rbp\n\t");
        asm!(self, "mov %rsp, %rbp\n\t");
    }

    fn end_main(&mut self) {
        asm!(self, "mov %rbp, %rsp\n\t");
        asm!(self, "pop %rbp\n\t");
        asm!(self, "ret\n");
    }

    fn expr(&mut self, expr: &Expr) -> Result<(), Error> {
        match expr.kind {
            ExprKind::Lit(WithSpan {
                value: Lit::Num(num),
                ..
            }) => asm!(self, "mov ${}, %eax\n\t", num),
            ExprKind::Lit(..) => unimplemented!(),
            ExprKind::Var(i) => asm!(self, "mov -{}(%rbp), %eax\n\t", (i + 1) * 4),
            ExprKind::Binary(ref expr) => self.binary_op(expr)?,
            ExprKind::Call(ref call) => self.call(call)?,
        }

        Ok(())
    }

    fn call(&mut self, call: &Call) -> Result<(), Error> {
        const REGISTERS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

        for i in 1..call.args.len() {
            asm!(self, "push %{}\n\t", REGISTERS[i]);
        }

        for arg in &call.args {
            self.expr(arg)?;
            asm!(self, "push %rax\n\t");
        }

        for i in 0..call.args.len() {
            asm!(self, "pop %{}\n\t", REGISTERS[i]);
        }

        asm!(self, "mov $0, %eax\n\t");
        asm!(self, "call {}\n\t", call.name);

        for i in 1..call.args.len() {
            asm!(self, "pop %{}\n\t", REGISTERS[i]);
        }

        Ok(())
    }

    fn binary_op(&mut self, expr: &BinaryExpr) -> Result<(), Error> {
        if let BinaryOp::Assign = expr.op.value {
            self.expr(&expr.right)?;

            match expr.left.kind {
                ExprKind::Var(i) => {
                    asm!(self, "mov %eax, -{}(%rbp)\n\t", (i + 1) * 4)
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
        asm!(self, "push %rax\n\t");
        self.expr(&expr.right)?;

        match expr.op.value {
            BinaryOp::Div => {
                asm!(self, "mov %eax, %ebx\n\t");
                asm!(self, "pop %rax\n\t");
                asm!(self, "mov $0, %edx\n\t");
                asm!(self, "idiv %ebx\n\t");
            }
            _ => {
                asm!(self, "pop %rbx\n\t");
                asm!(self, "{} %ebx, %eax\n\t", op);
            }
        }

        Ok(())
    }

    // fn string(&mut self, str: &str) -> Result<(), Error> {
    //     asm!(self, "\t.data\n");
    //     asm!(self, ".mydata:\n\nt");
    //     asm!(self, ".string \"");

    //     asm!(self, "{}", str);

    //     asm!(self, "\"\n\t");
    //     asm!(self, ".text\n\t");
    //     asm!(self, ".global stringfn\n");
    //     asm!(self, "stringfn:\n\t");
    //     asm!(self, "lea .mydata(%rip), %rax\n\t");
    //     asm!(self, "ret\n");

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

macro_rules! _asm {
    ($self:ident, $($tt:tt)*) => {
        std::write!($self.out, $($tt)*).expect("failed to write output")
    }
}

pub(self) use _asm as asm;
