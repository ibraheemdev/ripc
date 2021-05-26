use std::iter::Peekable;

fn main() {
    let tokens = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("error: invalid number of arguments");
        std::process::exit(1);
    });

    let mut tokens = tokens.chars().peekable();

    println!("  .globl main");
    println!("main:");

    println!("  mov ${}, %rax", take_digits(&mut tokens));

    while let Some(token) = tokens.next() {
        match token {
            '+' => {
                println!("  add ${}, %rax", take_digits(&mut tokens));
                continue;
            }
            '-' => {
                println!("  sub ${}, %rax", take_digits(&mut tokens));
                continue;
            }
            x => {
                eprintln!("error: unexpected character {}", x);
                std::process::exit(1);
            }
        }
    }

    println!("  ret");
}

fn take_digits(tokens: &mut Peekable<std::str::Chars>) -> u32 {
    let mut res = 0;
    while let Some(c) = tokens.peek() {
        match c.to_digit(10) {
            Some(i) => res = res * 10 + i,
            None => break,
        }
        tokens.next();
    }
    res
}
