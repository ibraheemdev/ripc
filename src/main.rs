use std::io::Write;

macro_rules! error {
    ($($arg:tt)*) => {{
        eprint!("error: ");
        eprint!($($arg)*);
        eprint!("\n");
        std::io::stdout().flush().unwrap();
        std::process::exit(1);
    }}
}

macro_rules! next_num {
    ($tokens:ident) => {
        match $tokens.next() {
            Some(Token::Num(num)) => num,
            _ => error!("expected number"),
        }
    };
}

fn main() {
    let input = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("error: invalid number of arguments");
        std::process::exit(1);
    });

    let mut tokens = Lexer::new(input);

    println!("  .globl main");
    println!("main:");

    println!("  mov ${}, %rax", next_num!(tokens));

    while let Some(token) = tokens.next() {
        match token {
            Token::Add => {
                println!("  add ${}, %rax", next_num!(tokens));
            }
            Token::Sub => {
                println!("  sub ${}, %rax", next_num!(tokens));
            }
            Token::Eof => {
                break;
            }
            _ => error!("invalid token"),
        }
    }

    println!("  ret");
}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Add,
    Sub,
    Num(usize),
    Eof,
}

struct Lexer {
    input: Vec<u8>,
    pos: usize,
    done: bool,
}

impl Lexer {
    fn new(input: String) -> Self {
        Self {
            input: input.into_bytes(),
            pos: 0,
            done: false,
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if self.done {
            return None;
        }

        while let Some(&byte) = self.input.get(self.pos) {
            match byte {
                b'+' => {
                    self.pos += 1;
                    return Some(Token::Add);
                }
                b'-' => {
                    self.pos += 1;
                    return Some(Token::Sub);
                }
                b'0'..=b'9' => {
                    let (num, len) = take_num(&self.input[self.pos..]);
                    self.pos += len;
                    return Some(Token::Num(num));
                }
                b if b.is_ascii_whitespace() => {
                    self.pos += 1;
                    continue;
                }
                _ => {
                    error!("invalid token");
                }
            }
        }

        self.done = true;
        Some(Token::Eof)
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
