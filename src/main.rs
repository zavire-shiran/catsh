use std::io;
use std::io::{Write};

fn main() {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    loop {
        stdout.write(b"$ ").is_ok();
        stdout.flush().is_ok();

        let mut input = String::new();
        match stdin.read_line(&mut input) {
            Ok(0) => break,
            _ => {}
        }

        stdout.write(input.as_bytes()).is_ok();
    }
}
