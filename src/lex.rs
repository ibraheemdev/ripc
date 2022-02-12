use crate::{Report, Reporter, Span, Spanned};

use std::fmt;
use std::io::Write;
use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Token<'a> {
    pub kind: TokenKind<'a>,
    pub span: Span,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenKind<'a> {
    Add,
    Sub,
    Num(usize),
    Str(&'a str),
    Whitespace,
    Eof,
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

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn peek_n(&self, n: usize) -> Option<char> {
        self.chars.clone().nth(n)
    }

    fn chomp(&mut self) -> Option<char> {
        self.chars.next().map(|x| {
            self.span.end += 1;
            x
        })
    }

    fn slice(&self) -> &'a str {
        &self.source[self.span.range().unwrap()]
    }

    fn reset(&mut self) {
        self.span.start = self.span.end;
    }

    fn chomp_while(&mut self, f: impl Fn(&char) -> bool + Copy) {
        while self.peek().map(|x| f(&x)).unwrap_or(false) {
            self.chomp();
        }
    }

    pub fn current_span(&self) -> Span {
        if self.eof {
            Span::EOF
        } else {
            self.span
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        use ErrorKind::*;
        use TokenKind::*;

        self.reset();
        if let Some(ch) = self.chomp() {
            let kind = match ch {
                '+' => Add,
                '-' => Sub,
                '0'..='9' => {
                    self.chomp_while(char::is_ascii_digit);
                    Num(self.slice().parse().unwrap())
                }
                ch if ch.is_ascii_whitespace() => {
                    self.chomp_while(char::is_ascii_whitespace);
                    Whitespace
                }
                '"' => loop {
                    match self.peek() {
                        Some('"') => {
                            self.chomp();
                            let str = self.slice();
                            break TokenKind::Str(&str[1..str.len() - 1]);
                        }
                        Some('\\') if matches!(self.peek_n(1), Some('\\') | Some('"')) => {
                            self.chomp();
                        }
                        Some(_) => {}
                        None => {
                            self.eof = true;
                            return Some(Err(Error::new(UnexpectedEof, self.span)));
                        }
                    }

                    self.chomp();
                },
                ch => return Some(Err(Error::new(InvalidCharacter(ch), self.span))),
            };

            if self.peek().is_none() {
                self.eof = true;
            }

            let token = Token {
                kind,
                span: self.span,
            };

            return Some(Ok(token));
        }

        self.eof = true;
        None
    }
}

impl<'a> fmt::Display for TokenKind<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TokenKind::Add => write!(f, "+"),
            TokenKind::Sub => write!(f, "-"),
            TokenKind::Num(num) => write!(f, "{}", num),
            TokenKind::Str(str) => write!(f, "{}", str),
            TokenKind::Whitespace => write!(f, " "),
            TokenKind::Eof => write!(f, "eof"),
        }
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
    UnexpectedEof,
    InvalidCharacter(char),
}

impl Spanned for Error {
    fn span(&self) -> Span {
        self.span
    }
}

impl<W: Write> Report<W> for Error {
    fn report(&self, f: &mut Reporter<'_, W>) -> std::io::Result<()> {
        match self.kind {
            ErrorKind::InvalidCharacter(ch) => write!(f.out, "Invalid character '{}'", ch),
            ErrorKind::UnexpectedEof => write!(f.out, "Found unexpected EOF"),
        }
    }
}
