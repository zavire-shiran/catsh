use std::io;
use std::io::{Write};

fn main() {
    let mut stdout = io::stdout();
    stdout.write(b"$ ").expect("");
    stdout.flush().expect("");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("");
    println!("{}", input);
}
