use crate::{ErrorReporter, Report, Span, Spanned};

use std::fmt;
use std::io::Write;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub span: Span,
}

impl LexError {
    fn new(kind: LexErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum LexErrorKind {
    InvalidCharacter(char),
}

impl Spanned for LexError {
    fn span(&self) -> Span {
        self.span
    }
}

impl<W: Write> Report<W> for LexError {
    fn report(&self, f: &mut ErrorReporter<'_, W>) -> std::io::Result<()> {
        match self.kind {
            LexErrorKind::InvalidCharacter(ch) => write!(f.writer, "Invalid character '{}'", ch),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind<'a> {
    Add,
    Sub,
    Mul,
    Div,
    OpenParen,
    CloseParen,
    Num(usize),
    Str(&'a str),
    Whitespace,
    Eof,
}

impl<'a> fmt::Display for TokenKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TokenKind::Add => write!(f, "+"),
            TokenKind::Sub => write!(f, "-"),
            TokenKind::Mul => write!(f, "*"),
            TokenKind::Div => write!(f, "/"),
            TokenKind::OpenParen => write!(f, "("),
            TokenKind::CloseParen => write!(f, ")"),
            TokenKind::Num(num) => write!(f, "{}", num),
            TokenKind::Str(str) => write!(f, "{}", str),
            TokenKind::Whitespace => write!(f, " "),
            TokenKind::Eof => write!(f, "eof"),
        }
    }
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    source: &'a str,
    span: Span,
    eof: bool,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            source,
            span: Span::default(),
            eof: false,
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn chomp(&mut self) -> Option<char> {
        self.span.end += 1;
        self.chars.next()
    }

    fn slice(&self) -> &'a str {
        &self.source[self.span.range()]
    }

    fn reset(&mut self) {
        self.span.start = self.span.end;
    }

    fn chomp_while(&mut self, f: impl Fn(&char) -> bool + Copy) {
        while self.peek().map(f).unwrap_or(false) {
            self.chomp();
        }
    }

    fn eof_span(&self) -> Span {
        Span::new(self.source.len() - 1..self.source.len())
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        use LexErrorKind::*;
        use TokenKind::*;

        self.reset();
        if let Some(ch) = self.chomp() {
            let kind = match ch {
                '+' => Add,
                '-' => Sub,
                '(' => OpenParen,
                ')' => CloseParen,
                '/' => Div,
                '*' => Mul,
                '0'..='9' => {
                    self.chomp_while(char::is_ascii_digit);
                    Num(self.slice().parse().unwrap())
                }
                ch if ch.is_ascii_whitespace() => {
                    self.chomp_while(char::is_ascii_whitespace);
                    Whitespace
                }
                ch => return Some(Err(LexError::new(InvalidCharacter(ch), self.span))),
            };

            let token = Token {
                kind,
                span: self.span,
            };
            return Some(Ok(token));
        }

        None
    }
}
