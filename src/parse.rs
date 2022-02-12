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
    Mul,
    Div,
}

impl BinaryOp {
    fn precedence(&self) -> usize {
        match self {
            BinaryOp::Sub => 1,
            BinaryOp::Add => 1,
            BinaryOp::Mul => 2,
            BinaryOp::Div => 2,
        }
    }
}

pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: WithSpan<BinaryOp>,
    pub right: Box<Expr>,
}

pub struct Parser<'a> {
    tokens: Tokens<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            tokens: Tokens {
                lexer,
                peeked: None,
            },
        }
    }

    fn peek(&mut self) -> Result<Option<Token<'a>>, lex::Error> {
        self.tokens.peek().copied().transpose()
    }

    fn next(&mut self) -> Result<Option<Token<'a>>, lex::Error> {
        self.tokens.next().transpose()
    }

    fn chomp(&mut self) {
        let _ = self.next().unwrap();
    }

    pub fn expr(&mut self) -> Result<Expr, Error> {
        self.expr_inner(0)
    }

    pub fn expr_inner(&mut self, precedence: usize) -> Result<Expr, Error> {
        let mut expr = self.primary()?;

        loop {
            let token = match self.peek()? {
                Some(t) => t,
                None => return Ok(expr),
            };

            let op = match token.kind {
                TokenKind::Add => BinaryOp::Add,
                TokenKind::Sub => BinaryOp::Sub,
                TokenKind::Mul => BinaryOp::Mul,
                TokenKind::Div => BinaryOp::Div,
                _ => return Err(Error::new(ErrorKind::ExpectedOperator, token.span)),
            };

            if op.precedence() < precedence {
                return Ok(expr);
            }

            self.chomp();

            let right = self.expr_inner(op.precedence() + 1)?;

            expr = Expr {
                span: expr.span + right.span,
                kind: ExprKind::Binary(BinaryExpr {
                    op: WithSpan::new(op, token.span),
                    left: Box::new(expr),
                    right: Box::new(right),
                }),
            };
        }
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let token = match self.next()? {
            Some(t) => t,
            None => {
                return Err(Error::new(
                    ErrorKind::ExpectedExpression,
                    self.tokens.current_span(),
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
                "Expected binary operator, found '{}'",
                self.span.range().map(|x| &f.source[x]).unwrap_or("EOF")
            ),
            ExpectedNumber => write!(f.out, "Expected number"),
            Lex(ref err) => err.report(f),
        }
    }
}

pub struct Tokens<'a> {
    lexer: Lexer<'a>,
    peeked: Option<Option<Result<Token<'a>, lex::Error>>>,
}

impl<'a> Tokens<'a> {
    pub fn peek(&mut self) -> Option<&Result<Token<'a>, lex::Error>> {
        let lexer = &mut self.lexer;
        self.peeked.get_or_insert_with(|| lexer.next()).as_ref()
    }
}

impl<'a> Iterator for Tokens<'a> {
    type Item = Result<Token<'a>, lex::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let token = match self.peeked.take() {
                Some(v) => v,
                None => self.lexer.next(),
            };

            match token {
                Some(Ok(Token {
                    kind: TokenKind::Whitespace,
                    ..
                })) => continue,
                t => break t,
            }
        }
    }
}

impl<'a> std::ops::Deref for Tokens<'a> {
    type Target = Lexer<'a>;

    fn deref(&self) -> &Self::Target {
        &self.lexer
    }
}
