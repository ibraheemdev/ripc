use std::error::Error as StdError;
use std::io::{self, Write};
use std::ops::Range;
use std::{fmt, str};

fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("invalid arguments");
        std::process::exit(1)
    });

    let mut reporter = ErrorReporter::new(io::stderr(), &input);

    match compile(&input) {
        Ok(()) => {}
        Err(e) => reporter.exit(e),
    }
}

struct ErrorReporter<'a, W> {
    writer: W,
    input: &'a str,
}

impl<'a, W: Write> ErrorReporter<'a, W> {
    fn new(writer: W, input: &'a str) -> Self {
        Self { writer, input }
    }

    fn report(&mut self, err: Error) -> Result<(), io::Error> {
        match err {
            Error::Lex(err) => self.report_spanned(err, err.span),
            Error::Parse(err) => self.report_spanned(err, err.span),
        }
    }

    fn report_spanned(&mut self, err: impl StdError, span: Span) -> Result<(), io::Error> {
        write!(self.writer, "[error]: {}\n", err)?;
        write!(self.writer, "{}\n", self.input)?;
        write!(self.writer, "{:space$}^ \n", "", space = span.start)
    }

    fn exit(&mut self, err: Error) -> ! {
        self.report(err).expect("failed to write to stdout");
        std::process::exit(1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Span {
    start: usize,
    end: usize,
}

impl Span {
    fn new(Range { start, end }: Range<usize>) -> Self {
        Self { start, end }
    }

    fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

#[derive(Debug)]
enum Error {
    Lex(LexError),
    Parse(ParseError),
}

impl From<LexError> for Error {
    fn from(err: LexError) -> Self {
        Self::Lex(err)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lex(ref err) => fmt::Display::fmt(err, f),
            Self::Parse(ref err) => fmt::Display::fmt(err, f),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct LexError {
    kind: LexErrorKind,
    span: Span,
}

impl LexError {
    fn new(kind: LexErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum LexErrorKind {
    InvalidCharacter(char),
}

impl StdError for LexError {}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            LexErrorKind::InvalidCharacter(ch) => write!(f, "Invalid character '{}'", ch),
        }
    }
}

struct Token {
    kind: TokenKind,
    span: Span,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum TokenKind {
    Add,
    Sub,
    Num(usize),
    Whitespace,
    Eof,
}

struct Lexer<'a> {
    peeked: Option<Option<char>>,
    chars: str::Chars<'a>,
    source: &'a str,
    start: usize,
    end: usize,
    eof: bool,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars(),
            source,
            peeked: None,
            start: 0,
            end: 0,
            eof: false,
        }
    }

    fn peek(&mut self) -> Option<&char> {
        let chars = &mut self.chars;
        self.peeked.get_or_insert_with(|| chars.next()).as_ref()
    }

    fn chomp(&mut self) -> Option<char> {
        self.end += 1;
        match self.peeked.take() {
            Some(v) => v,
            None => self.chars.next(),
        }
    }

    fn span(&self) -> Span {
        Span::new(self.start..self.end)
    }

    fn chomp_while(&mut self, f: impl Fn(&char) -> bool + Copy) {
        while self.peek().map(f).unwrap_or(false) {
            self.chomp();
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        use LexErrorKind::*;
        use TokenKind::*;

        self.start = self.end;
        if let Some(ch) = self.chomp() {
            let kind = match ch {
                '+' => Add,
                '-' => Sub,
                '0'..='9' => {
                    self.chomp_while(|x| x.is_ascii_digit());
                    self.source[self.span().range()]
                        .parse::<usize>()
                        .map(Num)
                        .unwrap()
                }
                ch if ch.is_ascii_whitespace() => {
                    self.chomp_while(|x| x.is_ascii_whitespace());
                    Whitespace
                }
                ch => return Some(Err(LexError::new(InvalidCharacter(ch), self.span()))),
            };

            let token = Token {
                kind,
                span: self.span(),
            };
            return Some(Ok(token));
        }

        self.eof = true;
        let token = Token {
            kind: TokenKind::Eof,
            span: self.span(),
        };
        return Some(Ok(token));
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct ParseError {
    kind: ParseErrorKind,
    span: Span,
}

impl ParseError {
    fn new(kind: ParseErrorKind, span: Span) -> Self {
        Self { kind, span }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ParseErrorKind {
    ExpectedNumber,
    InvalidToken,
}

impl StdError for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ParseErrorKind::ExpectedNumber => write!(f, "Expected number"),
            ParseErrorKind::InvalidToken => write!(f, "Invalid token"),
        }
    }
}

struct Parser<'a> {
    tokens: Lexer<'a>,
}

impl<'a> Parser<'a> {
    fn should_skip(tok: &TokenKind) -> bool {
        matches!(tok, TokenKind::Whitespace)
    }

    fn next_num(&mut self) -> Result<usize, Error> {
        loop {
            let token = self.tokens.next().transpose()?;
            match token.map(|x| x.kind) {
                Some(TokenKind::Num(num)) => return Ok(num),
                Some(ref x) if Self::should_skip(x) => continue,
                Some(_) => {
                    return Err(Error::Parse(ParseError::new(
                        ParseErrorKind::ExpectedNumber,
                        self.tokens.span(),
                    )))
                }
                None => {
                    return Err(Error::Parse(ParseError::new(
                        ParseErrorKind::ExpectedNumber,
                        self.tokens.span(),
                    )))
                }
            }
        }
    }
}

fn compile<'a>(input: &'a str) -> Result<(), Error> {
    use ParseErrorKind::*;
    use TokenKind::*;

    let tokens = Lexer::new(input);
    let mut parser = Parser { tokens };

    println!("  .globl main");
    println!("main:");

    println!("  mov ${}, %rax", parser.next_num()?);

    while let Some(token) = parser.tokens.next() {
        let token = token?;

        match token.kind {
            Whitespace => continue,
            Add => {
                println!("  add ${}, %rax", parser.next_num()?);
            }
            Sub => {
                println!("  sub ${}, %rax", parser.next_num()?);
            }
            Eof => break,
            _ => return Err(Error::Parse(ParseError::new(InvalidToken, token.span))),
        }
    }

    println!("  ret");
    Ok(())
}
