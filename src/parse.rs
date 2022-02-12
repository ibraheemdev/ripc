use crate::lex::{self, Lexer, Token, TokenKind};
use crate::{Report, Reporter, Span, Spanned, WithSpan};

use std::io::Write;

pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

pub enum ExprKind {
    Lit(WithSpan<Lit>),
    Binary(BinaryExpr),
}

pub enum Lit {
    Num(usize),
    String(String),
}

pub enum BinaryOp {
    Sub,
    Add,
}

pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: WithSpan<BinaryOp>,
    pub right: Box<Expr>,
}

pub struct Parser<'a> {
    lexer: SkipWhiteSpace<'a>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: SkipWhiteSpace { lexer },
            pos: 0,
        }
    }

    fn next(&mut self) -> Result<Option<Token<'a>>, lex::Error> {
        self.pos += 1;
        self.lexer.next().transpose()
    }

    pub fn expr(&mut self) -> Result<Expr, Error> {
        let primary = self.primary()?;
        self.expr_inner(primary)
    }

    pub fn expr_inner(&mut self, expr: Expr) -> Result<Expr, Error> {
        let token = match self.next()? {
            Some(t) => t,
            None => return Ok(expr),
        };

        let op = match token.kind {
            TokenKind::Add => BinaryOp::Add,
            TokenKind::Sub => BinaryOp::Sub,
            _ => return Err(Error::new(ErrorKind::ExpectedOperator, token.span)),
        };

        let right = self.primary()?;

        self.expr_inner(Expr {
            span: expr.span + right.span,
            kind: ExprKind::Binary(BinaryExpr {
                op: WithSpan::new(op, token.span),
                left: Box::new(expr),
                right: Box::new(right),
            }),
        })
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let token = match self.next()? {
            Some(t) => t,
            None => {
                return Err(Error::new(
                    ErrorKind::ExpectedExpression,
                    self.lexer.current_span(),
                ));
            }
        };

        match token.kind {
            TokenKind::Num(num) => Ok(Expr {
                kind: ExprKind::Lit(WithSpan::new(Lit::Num(num), token.span)),
                span: token.span,
            }),
            TokenKind::Str(str) => Ok(Expr {
                kind: ExprKind::Lit(WithSpan::new(Lit::String(str.to_owned()), token.span)),
                span: token.span,
            }),
            _ => Err(Error::new(ErrorKind::ExpectedExpression, token.span)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
}

impl From<lex::Error> for Error {
    fn from(err: lex::Error) -> Self {
        Self {
            kind: ErrorKind::Lex(err),
            span: err.span,
        }
    }
}

impl Error {
    fn new(kind: ErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ErrorKind {
    ExpectedNumber,
    ExpectedOperator,
    ExpectedExpression,
    Lex(lex::Error),
}

impl Spanned for Error {
    fn span(&self) -> Span {
        self.span
    }
}

impl<W: Write> Report<W> for Error {
    fn report(&self, f: &mut Reporter<'_, W>) -> std::io::Result<()> {
        use ErrorKind::*;

        match self.kind {
            ExpectedExpression => write!(
                f.out,
                "Expected expression, found '{}'",
                self.span.range().map(|x| &f.source[x]).unwrap_or("EOF")
            ),
            ExpectedOperator => write!(
                f.out,
                "Expected expression, found '{}'",
                self.span.range().map(|x| &f.source[x]).unwrap_or("EOF")
            ),
            ExpectedNumber => write!(f.out, "Expected number"),
            Lex(ref err) => err.report(f),
        }
    }
}

pub struct SkipWhiteSpace<'a> {
    lexer: Lexer<'a>,
}

impl<'a> Iterator for SkipWhiteSpace<'a> {
    type Item = Result<Token<'a>, lex::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.lexer.next() {
                Some(Ok(Token {
                    kind: TokenKind::Whitespace,
                    ..
                })) => continue,
                t => break t,
            }
        }
    }
}

impl<'a> std::ops::Deref for SkipWhiteSpace<'a> {
    type Target = Lexer<'a>;

    fn deref(&self) -> &Self::Target {
        &self.lexer
    }
}
