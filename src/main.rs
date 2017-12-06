use std::io;
use std::io::{Write};

fn main() {
    let mut stdout = io::stdout();

    loop {
        stdout.write(b"$ ").expect("");
        stdout.flush().expect("");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("");
        if input.len() == 0 {
            stdout.write(b"\n").is_ok();
            break
        }
        stdout.write(input.as_bytes()).expect("");
    }
}
