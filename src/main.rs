use std::env;
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

        let split_line = split(input);

        println!("{:?}", split_line);

        if split_line.len() > 0 {
            execute_command(split_line);
        }
    }
}

fn split(line: String) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();

    for word in line.split_whitespace() {
        ret.push(word.to_string());
    }

    return ret;
}

fn execute_command(command: Vec<String>) {
    match &command[0][..] {
        "cd" => {
            if command.len() > 1 {
                env::set_current_dir(&command[1]).expect("");
            }
        }
        _ => { fork_exec(command) }
    }
}

fn fork_exec(command: Vec<String>) {

}
