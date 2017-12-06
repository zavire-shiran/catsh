use std::io;
use std::io::{Write};

fn main() {
    let mut stdout = io::stdout();
    stdout.write(b"$ ");
    stdout.flush();
    let mut input = String::new();
    io::stdin().read_line(&mut input);
    println!("{}", input);
}
