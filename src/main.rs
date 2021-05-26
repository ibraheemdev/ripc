use std::collections::VecDeque;
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

    let mut tokens = lex(input);

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

#[derive(Debug)]
struct Tokens(VecDeque<Token>);

impl Tokens {
    fn next(&mut self) -> Option<Token> {
        self.0.pop_front()
    }
}

fn lex(input: String) -> Tokens {
    let mut tokens = VecDeque::new();
    let bytes = input.into_bytes();

    let mut i = 0;
    while let Some(&byte) = bytes.get(i) {
        match byte {
            b'+' => {
                tokens.push_back(Token::Add);
                i += 1;
                continue;
            }
            b'-' => {
                tokens.push_back(Token::Sub);
                i += 1;
                continue;
            }
            b'0'..=b'9' => {
                let (num, len) = take_num(&bytes[i..]);
                tokens.push_back(Token::Num(num));
                i += len;
                continue;
            }
            b if b.is_ascii_whitespace() => {
                i += 1;
                continue;
            }
            _ => {
                error!("invalid token");
            }
        }
    }

    tokens.push_back(Token::Eof);
    return Tokens(tokens);
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
