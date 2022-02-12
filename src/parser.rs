use crate::lexer::{LexError, Lexer, Token, TokenKind};
use crate::{ErrorReporter, Report, Span, Spanned};

use std::io::Write;
use std::iter::Peekable;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub span: Span,
}

impl From<LexError> for ParseError {
    fn from(err: LexError) -> Self {
        Self {
            kind: ParseErrorKind::Lex(err),
            span: err.span,
        }
    }
}

impl ParseError {
    fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span }
    }

    fn unexpected_eof(expected: String) -> Self {
        Self {
            kind: ParseErrorKind::UnexpectedEof(expected),
            span: Span::new(0..0),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ParseErrorKind {
    Lex(LexError),
    ExpectedNumber,
    InvalidToken,
    UnexpectedToken(String),
    UnexpectedEof(String),
    MissingClosingParenFor,
    ExpectedExpression,
}

impl Spanned for ParseError {
    fn span(&self) -> Span {
        self.span
    }
}

impl<W: Write> Report<W> for ParseError {
    fn report(&self, f: &mut ErrorReporter<'_, W>) -> std::io::Result<()> {
        use ParseErrorKind::*;

        match self.kind {
            Lex(ref err) => err.report(f),
            ExpectedExpression => write!(
                f.writer,
                "Expected expression, found '{}'",
                &f.source[self.span.range()]
            ),
            ExpectedNumber => write!(f.writer, "Expected number"),
            InvalidToken => write!(f.writer, "Invalid token"),
            UnexpectedEof(expected) => {
                write!(f.writer, "expected {}, found end of file", expected)
            }
            UnexpectedToken(expected) => {
                write!(
                    f.writer,
                    "expected {}, found '{}'",
                    expected,
                    &f.source[self.span.range()]
                )
            }
            MissingClosingParen => write!(f.writer, "Missing closing parentheses"),
        }
    }
}

type BoxExpr = Box<Expr>;

pub enum Expr {
    Lit(Lit),
    Binary(BinaryExpr),
}

pub enum Lit {
    Num(usize),
}

pub enum BinaryOp {
    Sub,
    Div,
    Mul,
    Add,
}

pub struct BinaryExpr {
    left: BoxExpr,
    op: BinaryOp,
    right: BoxExpr,
}

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: lexer.peekable(),
            pos: 0,
        }
    }

    fn should_skip(tok: TokenKind<'a>) -> bool {
        matches!(tok, TokenKind::Whitespace)
    }

    fn next(&mut self) -> Result<Option<Token<'a>>, LexError> {
        self.pos += 1;
        self.lexer.next().transpose()
    }

    fn peek(&mut self) -> Result<Option<Token<'a>>, LexError> {
        self.lexer.peek().copied().transpose()
    }

    fn next_num(&mut self) -> Result<usize, ParseError> {
        use ParseErrorKind::*;

        match self.next()? {
            Some(Token {
                kind: TokenKind::Num(num),
                span,
            }) => Ok(num),
            Some(token) => Err(ParseError::new(ExpectedNumber, token.span)),
            None => Err(ParseError::new(
                UnexpectedToken("number".to_owned()),
                Span::new(0..0),
            )),
        }
    }

    pub fn expr(&self) -> Result<Expr, ParseError> {
        let expr = self.mul_div();
    }

    pub fn mul_div(&self) -> Result<Expr, ParseError> {
        let expr = self.primary();
    }

    fn primary(&self) -> Result<Expr, ParseError> {
        let token = self.next()?;
        match token.map(|t| t.kind) {
            Some(TokenKind::OpenParen) => {
                let expr = self.expr();
                match self.next()?.map(|t| t.kind) {
                    Some(TokenKind::OpenParen) => {}
                    Some(_) | None => {
                        return Err(ParseError::new(
                            ParseErrorKind::MissingClosingParenFor,
                            token.unwrap().span,
                        ))
                    }
                };
                expr
            }
            Some(TokenKind::Num(num)) => Ok(Expr::Lit(Lit::Num(num))),
            Some(_) => Err(ParseError::new(
                ParseErrorKind::ExpectedExpression,
                token.unwrap().span,
            )),
            None => Err(ParseError::unexpected_eof("expression".to_owned())),
        }
    }
}
