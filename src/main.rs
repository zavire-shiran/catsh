use std::io;
use std::io::{Write};

fn main() {
    let mut stdout = io::stdout();

    loop {
        stdout.write(b"$ ").is_ok();
        stdout.flush().is_ok();

        let mut input = String::new();
        io::stdin().read_line(&mut input).is_ok();
        if input.len() == 0 {
            stdout.write(b"\n").is_ok();
            break
        }
        stdout.write(input.as_bytes()).is_ok();
    }
}
