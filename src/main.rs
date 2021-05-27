use std::io::{self, Write};
use std::rc::Rc;
use std::{fmt, slice, str};

fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("invalid arguments");
        std::process::exit(1)
    });

    let input = Rc::new(input);

    let mut reporter = ErrorReporter::new(io::stderr(), input.clone());

    match compile(input.clone()) {
        Ok(()) => {}
        Err(e) => reporter.exit(e),
    }
}

struct ErrorReporter<W> {
    writer: W,
    input: Rc<String>,
}

impl<W: Write> ErrorReporter<W> {
    fn new(writer: W, input: Rc<String>) -> Self {
        Self { writer, input }
    }

    fn report(&mut self, err: Error) -> Result<(), io::Error> {
        match err {
            Error::Lex(err) => self.report_spanned(err),
            Error::Parse(err) => self.report_spanned(err),
        }
    }

    fn report_spanned(&mut self, err: Spannned<impl fmt::Display>) -> Result<(), io::Error> {
        write!(self.writer, "[error]: {}\n", err.inner.as_ref().unwrap())?;
        write!(self.writer, "{}\n", &*self.input)?;
        write!(self.writer, "{:rep$}^ \n", "", rep = err.start)
    }

    fn exit(&mut self, err: Error) -> ! {
        self.report(err).expect("failed to write to stdout");
        std::process::exit(1)
    }
}

struct Spannned<T> {
    inner: Option<T>,
    start: usize,
    _end: usize,
}

impl<T> Spannned<T> {
    fn new(range: std::ops::Range<usize>) -> Self {
        Self {
            inner: None,
            start: range.start,
            _end: range.end,
        }
    }

    fn with(mut self, inner: T) -> Self {
        self.inner = Some(inner);
        self
    }
}

enum Error {
    Lex(Spannned<LexError>),
    Parse(Spannned<ParseError>),
}

impl From<Spannned<LexError>> for Error {
    fn from(err: Spannned<LexError>) -> Self {
        Self::Lex(err)
    }
}

impl From<Spannned<ParseError>> for Error {
    fn from(err: Spannned<ParseError>) -> Self {
        Self::Parse(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lex(ref err) => fmt::Display::fmt(err.inner.as_ref().unwrap(), f),
            Self::Parse(ref err) => fmt::Display::fmt(err.inner.as_ref().unwrap(), f),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LexError {
    InvalidCharacter(u8),
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCharacter(ch) => write!(
                f,
                "Invalid character '{}'",
                str::from_utf8(slice::from_ref(ch)).unwrap()
            ),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Add,
    Sub,
    Num(usize),
    Eof,
}

struct Lexer {
    input: Rc<String>,
    pos: usize,
    done: bool,
}

impl Lexer {
    fn new(input: Rc<String>) -> Self {
        Self {
            input,
            pos: 0,
            done: false,
        }
    }
}

impl Iterator for Lexer {
    type Item = Result<Token, Spannned<LexError>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        while let Some(&byte) = self.input.as_bytes().get(self.pos) {
            let some = match byte {
                b'+' => {
                    self.pos += 1;
                    Ok(Token::Add)
                }
                b'-' => {
                    self.pos += 1;
                    Ok(Token::Sub)
                }
                b'0'..=b'9' => {
                    let (num, len) = take_num(&self.input.as_bytes()[self.pos..]);
                    self.pos += len;
                    Ok(Token::Num(num))
                }
                b if b.is_ascii_whitespace() => {
                    self.pos += 1;
                    continue;
                }
                b => Err(Spannned::new(self.pos..self.pos + 1).with(LexError::InvalidCharacter(b))),
            };
            return Some(some);
        }

        self.done = true;
        Some(Ok(Token::Eof))
    }
}

fn take_num(bytes: &[u8]) -> (usize, usize) {
    let mut val = 0_usize;
    let mut len = 0;

    for &byte in bytes {
        match byte_to_ascii_digit(byte) {
            Some(digit) => {
                val = val * 10 + digit as usize;
                len += 1;
            }
            None => break,
        }
    }
    (val, len)
}

fn byte_to_ascii_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        _ => None,
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParseError {
    ExpectedNumber,
    InvalidToken,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExpectedNumber => write!(f, "Expected number"),
            Self::InvalidToken => write!(f, "Invalid token"),
        }
    }
}

struct Parser {
    tokens: Lexer,
}

impl Parser {
    fn next_num(&mut self) -> Result<usize, Error> {
        let start = self.tokens.pos;
        match self.tokens.next() {
            Some(Err(e)) => Err(e.into()),
            Some(Ok(Token::Num(num))) => Ok(num),
            _ => {
                let end = self.tokens.pos;
                Err(Spannned::new(start..end)
                    .with(ParseError::ExpectedNumber)
                    .into())
            }
        }
    }
}

fn compile(input: Rc<String>) -> Result<(), Error> {
    let tokens = Lexer::new(input);
    let mut parser = Parser { tokens };

    println!("  .globl main");
    println!("main:");

    println!("  mov ${}, %rax", parser.next_num()?);

    while let Some(token) = parser.tokens.next() {
        let start = parser.tokens.pos;
        match token? {
            Token::Add => {
                println!("  add ${}, %rax", parser.next_num()?);
            }
            Token::Sub => {
                println!("  sub ${}, %rax", parser.next_num()?);
            }
            Token::Eof => {
                break;
            }
            _ => {
                let end = parser.tokens.pos;
                return Err(Error::Parse(
                    Spannned::new(start..end).with(ParseError::InvalidToken),
                ));
            }
        }
    }

    println!("  ret");
    Ok(())
}
