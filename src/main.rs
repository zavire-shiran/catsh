use std::env;
use std::io;
use std::io::{Write};
use std::path::Path;
use std::ffi::CString;

extern crate nix;
extern crate libc;

fn main() {
    let mut command_parser = CommandParser::new();

    loop {
        match command_parser.get_next_command() {
            Some(command) => execute_command(command),
            None => break
        }
    }
}

struct CommandParser {
    input_buffer: String,
    command_buffer: Vec<Vec<String>>
}

#[derive(PartialEq)]
enum ParserStatus {
    EOF,
    Ok
}

impl CommandParser {
    fn new() -> CommandParser {
        return CommandParser{
            input_buffer: std::string::String::new(),
            command_buffer: Vec::new() }
    }

    fn get_next_command(&mut self) -> Option<Vec<String>> {
        while self.command_buffer.len() == 0 {
            if self.parse_input() == ParserStatus::EOF {
                return None
            }
        }

        return Some(self.command_buffer.remove(0));
    }

    fn parse_input(&mut self) -> ParserStatus {
        let mut stdout = io::stdout();
        let stdin = io::stdin();

        stdout.write(b"$ ").is_ok();
        stdout.flush().is_ok();

        let mut input = String::new();
        return match stdin.read_line(&mut input) {
            Ok(0) => ParserStatus::EOF, // this always mean EOF, i think
            _ => {
                let command = parse_command(input);
                self.command_buffer.push(command);
                ParserStatus::Ok
            }
        }
    }
}


fn parse_command(line: String) -> Vec<String> {
    let mut ret: Vec<String> = Vec::new();

    for word in line.split_whitespace() {
        ret.push(word.to_string());
    }

    return ret;
}

fn execute_command(command: Vec<String>) {
    if command.len() == 0 {
        return;
    }
    match &command[0][..] {
        "cd" => {
            if command.len() > 1 {
                env::set_current_dir(&command[1]).expect("");
                println!("{:?}", env::current_dir());
            } else {
                match env::var("HOME") {
                    Ok(home_path) => env::set_current_dir(home_path).expect(""),
                    Err(_) => println!("Don't know where home is :/")
                }
            }
        }
        _ => { run_command_line(command) }
    }
}

fn run_command_line(command: Vec<String>) {
    let command_name = &command[0];
    let command_args: Vec<CString> = command.iter().map(|ref s| CString::new(s.as_bytes()).unwrap()).collect();

    if command_name.starts_with('.') {
        if Path::new(&command_name).exists() {
            fork_exec(&CString::new(command_name.as_bytes()).unwrap(), &command_args);
        } else {
            println!("Could not file command {}", command_name);
        }
    } else {
        match get_path_for_command(command_name) {
            Some(command_path) => { fork_exec(&command_path, &command_args); }
            None => { println!("Could not find command {}", command_name); }
        }
    }
}

fn get_path_for_command(command_name: &String) -> Option<CString> {
    let path_list = get_path_list();

    for path_str in path_list {
        let path = Path::new(&path_str);
        let command_path = path.join(command_name);
        if command_path.exists() {
            match command_path.to_str() {
                Some(cp) => { return Some(CString::new(cp).unwrap()); }
                None => ()
            }
        }
    }
    return None;
}

fn get_path_list() -> Vec<String> {
    match std::env::var("PATH") {
        Ok(path) => {
            let p: Vec<String> = path.split(':').map(|x| x.to_string()).collect();
            return p;
        }
        Err(_) => {
            println!("Warning: could not retrive $PATH");
            return vec!["/bin".into(), "/usr/bin".into()];
        }
    }
}

fn fork_exec(command_name: &CString, command_args: &[CString]) {
    use nix::unistd::{fork, ForkResult};

    match fork() {
        Ok(ForkResult::Parent {child, ..}) => {
            nix::sys::wait::waitpid(child, None).expect("waitpid failed");
        }
        Ok(ForkResult::Child) => {
            nix::unistd::execv(command_name, command_args).expect("exec failed");
        }
        Err(_) => { println!("fork failed") }
    }
}
