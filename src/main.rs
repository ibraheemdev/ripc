fn main() {
    let arg = std::env::args()
        .nth(1)
        .map(|x| x.parse::<usize>().ok())
        .flatten();

    match arg {
        Some(num) => {
            println!("  .globl main");
            println!("main:");
            println!("mov ${}, %rax", num);
            println!("  ret");
        }
        None => {
            eprintln!("error: invalid number of arguments");
            std::process::exit(1);
        }
    }
}
