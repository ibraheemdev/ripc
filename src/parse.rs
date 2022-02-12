use crate::lex::{self, Lexer, Token, TokenKind};
use crate::{Report, Reporter, Span, Spanned, WithSpan};

use std::io::Write;
use std::mem;

pub struct Parser<'a> {
    tokens: Tokens<'a>,
    vars: Vec<Var>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            tokens: Tokens {
                lexer,
                peeked: None,
            },
            vars: Vec::new(),
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

    pub fn parse(&mut self) -> Result<Ast, Error> {
        let mut exprs = Vec::new();

        while let Some(expr) = self.expr(0)? {
            let token = self.next()?;

            if !matches!(
                token,
                Some(Token {
                    kind: TokenKind::Semi,
                    ..
                }),
            ) {
                return Err(Error::new(
                    ErrorKind::UnterminatedExpression,
                    token.map(|t| t.span).unwrap_or(Span::EOF),
                ));
            }

            exprs.push(expr);
        }

        Ok(Ast {
            exprs,
            vars: mem::take(&mut self.vars),
        })
    }

    pub fn expr(&mut self, precedence: usize) -> Result<Option<Expr>, Error> {
        let mut expr = match self.primary()? {
            Some(e) => e,
            None => return Ok(None),
        };

        loop {
            let token = match self.peek()? {
                Some(t) => t,
                None => return Ok(Some(expr)),
            };

            let op = match token.kind {
                TokenKind::Add => BinaryOp::Add,
                TokenKind::Sub => BinaryOp::Sub,
                TokenKind::Mul => BinaryOp::Mul,
                TokenKind::Div => BinaryOp::Div,
                TokenKind::Assign => BinaryOp::Assign,
                TokenKind::Semi => return Ok(Some(expr)),
                _ => return Err(Error::new(ErrorKind::ExpectedOperator, token.span)),
            };

            if op.precedence() < precedence {
                return Ok(Some(expr));
            }

            self.chomp();

            let right = self
                .expr(op.precedence() + 1)?
                .ok_or_else(|| Error::new(ErrorKind::UnexpectedEof, Span::EOF))?;

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

    fn primary(&mut self) -> Result<Option<Expr>, Error> {
        let token = match self.next()? {
            Some(t) => t,
            None => {
                return Ok(None);
            }
        };

        let kind = match token.kind {
            TokenKind::Num(num) => ExprKind::Lit(WithSpan::new(Lit::Num(num), token.span)),
            TokenKind::Str(lit) => {
                ExprKind::Lit(WithSpan::new(Lit::String(lit.to_owned()), token.span))
            }
            TokenKind::Ident(var) => {
                let i = self
                    .vars
                    .iter()
                    .position(|v| v.name == var)
                    .unwrap_or_else(|| {
                        self.vars.push(Var {
                            name: var.to_owned(),
                        });
                        self.vars.len() - 1
                    });

                ExprKind::Var(i)
            }
            _ => return Err(Error::new(ErrorKind::ExpectedExpression, token.span)),
        };

        Ok(Some(Expr {
            kind,
            span: token.span,
        }))
    }
}

pub struct Ast {
    pub exprs: Vec<Expr>,
    pub vars: Vec<Var>,
}

pub struct Var {
    name: String,
}

pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
}

pub enum ExprKind {
    Lit(WithSpan<Lit>),
    Binary(BinaryExpr),
    Var(usize),
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
    Assign,
}

impl BinaryOp {
    fn precedence(&self) -> usize {
        match self {
            BinaryOp::Assign => 1,
            BinaryOp::Sub | BinaryOp::Add => 2,
            BinaryOp::Mul | BinaryOp::Div => 3,
        }
    }
}

pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: WithSpan<BinaryOp>,
    pub right: Box<Expr>,
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
    UnexpectedEof,
    UnterminatedExpression,
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
            UnexpectedEof => write!(f.out, "Unexpected EOF"),
            UnterminatedExpression => write!(f.out, "Unterminated expression"),
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
        if let Some(ref token) = self.peeked {
            return token.as_ref();
        }

        loop {
            match self.lexer.next() {
                Some(Ok(Token {
                    kind: TokenKind::Whitespace,
                    ..
                })) => continue,
                t => {
                    self.peeked.replace(t);
                    break self.peeked.as_ref().unwrap().as_ref();
                }
            }
        }
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
