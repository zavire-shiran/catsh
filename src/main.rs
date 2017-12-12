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
        match command_parser.get_next_command_list() {
            Some(command_list) => execute_command_list(command_list),
            None => break
        }
    }
}

#[derive(PartialEq)]
enum RunConditions {
    Always,
    IfTrue,
    IfFalse
}

struct Command {
    arguments: Vec<String>,
    run_conditions: RunConditions
}

impl Command {
    fn always() -> Command {
        return Command {
            arguments: Vec::new(),
            run_conditions: RunConditions::Always
        }
    }

    fn if_true() -> Command {
        return Command {
            arguments: Vec::new(),
            run_conditions: RunConditions::IfTrue
        }
    }

    fn if_false() -> Command {
        return Command {
            arguments: Vec::new(),
            run_conditions: RunConditions::IfFalse
        }
    }

    fn push_argument(&mut self, arg: String) {
        self.arguments.push(arg);
    }

    fn len(&self) -> usize {
        return self.arguments.len();
    }
}

type CommandList = Vec<Command>;

struct CommandParser {
    input_buffer: String,
    command_list_buffer: Vec<CommandList>
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
            command_list_buffer: Vec::new() }
    }

    fn get_next_command_list(&mut self) -> Option<CommandList> {
        while self.command_list_buffer.len() == 0 {
            if self.parse_input() == ParserStatus::EOF {
                return None
            }
        }

        return Some(self.command_list_buffer.remove(0));
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
                let command_list = self.parse_command_list(input);
                self.command_list_buffer.push(command_list);
                ParserStatus::Ok
            }
        }
    }

    fn parse_command_list(&mut self, line: String) -> CommandList {
        let mut command_list: CommandList = Vec::new();
        let mut command: Command = Command::always();

        let tokens = tokenize_command(line);
        println!("{:?}", tokens);
        for token in tokens {
            if token.class == CommandLineTokenType::Argument {
                command.push_argument(token.lexeme);
            } else if command.len() == 0 {
                //this is a syntax error, but i don't want to deal with the Result<> right now
            } else if token.class == CommandLineTokenType::Semicolon {
                command_list.push(command);
                command = Command::always();
            } else if token.class == CommandLineTokenType::EOL {
                command_list.push(command);
                return command_list;
            } else if token.class == CommandLineTokenType::AndOp {
                command_list.push(command);
                command = Command::if_true();
            } else if token.class == CommandLineTokenType::OrOp {
                command_list.push(command);
                command = Command::if_false();
            }
        }

        return command_list;
    }

}

#[derive(Debug, PartialEq)]
enum CommandLineTokenType {
    Argument,
    EOL,
    Ampersand,
    Pipe,
    AndOp,
    OrOp,
    Semicolon
}

#[derive(Debug)]
struct CommandLineToken {
    class: CommandLineTokenType,
    lexeme: String
}

impl CommandLineToken {
    fn argument(lexeme: String) -> CommandLineToken {
        return CommandLineToken {
            class: CommandLineTokenType::Argument,
            lexeme: lexeme
        }
    }

    fn eol() -> CommandLineToken {
        return CommandLineToken {
            class: CommandLineTokenType::EOL,
            lexeme: String::from("\n")
        }
    }

    fn semicolon() -> CommandLineToken {
        return CommandLineToken {
            class: CommandLineTokenType::Semicolon,
            lexeme: String::from(";")
        }
    }

    fn and_op() -> CommandLineToken {
        return CommandLineToken {
            class: CommandLineTokenType::AndOp,
            lexeme: String::from("&&")
        }
    }

    fn or_op() -> CommandLineToken {
        return CommandLineToken {
            class: CommandLineTokenType::OrOp,
            lexeme: String::from("||")
        }
    }

    fn ampersand() -> CommandLineToken {
        return CommandLineToken {
            class: CommandLineTokenType::Ampersand,
            lexeme: String::from("&")
        }
    }

    fn pipe() -> CommandLineToken {
        return CommandLineToken {
            class: CommandLineTokenType::Pipe,
            lexeme: String::from("|")
        }
    }
}

fn tokenize_command(line: String) -> Vec<CommandLineToken> {
    let mut cur_arg_buf = String::new();
    let mut tokens = Vec::new();

    for c in line.chars() {
        if cur_arg_buf == "&" {
            if c == '&' {
                tokens.push(CommandLineToken::and_op());
                cur_arg_buf = String::new();
                continue;
            } else {
                tokens.push(CommandLineToken::ampersand());
                cur_arg_buf = String::new();
            }
        } else if cur_arg_buf == "|" {
            if c == '|' {
                tokens.push(CommandLineToken::or_op());
                cur_arg_buf = String::new();
                continue;
            } else {
                tokens.push(CommandLineToken::pipe());
                cur_arg_buf = String::new();
            }
        }


        if c.is_whitespace() {
            if cur_arg_buf.len() > 0 {
                tokens.push(CommandLineToken::argument(cur_arg_buf));
                cur_arg_buf = String::new();
            }
            if c == '\n' {
                tokens.push(CommandLineToken::eol())
            }
        } else if c == ';' {
            if cur_arg_buf.len() > 0 {
                tokens.push(CommandLineToken::argument(cur_arg_buf));
                cur_arg_buf = String::new();
            }
            tokens.push(CommandLineToken::semicolon());
        } else if c == '&' || c == '|' {
            if cur_arg_buf.len() > 0 {
                tokens.push(CommandLineToken::argument(cur_arg_buf));
                cur_arg_buf = String::new();
            }
            cur_arg_buf.push(c);
        } else  {
            cur_arg_buf.push(c);
        }
    }

    return tokens;
}

fn execute_command_list(command_list: CommandList) {
    for command in command_list {
        let status = execute_command(command.arguments);
        println!("command status: {}", status);
    }
}

fn execute_command(command: Vec<String>) -> i8 {
    if command.len() == 0 {
        return 0;
    }
    match &command[0][..] {
        "cd" => {
            if command.len() > 1 {
                env::set_current_dir(&command[1]).expect(""); // need to set command status on error here, so as not to panic
                println!("{:?}", env::current_dir());
                return 0;
            } else {
                match env::var("HOME") {
                    Ok(home_path) => {
                        env::set_current_dir(home_path).expect("");
                        return 0;
                    },
                    Err(_) => {
                        println!("Don't know where home is :/");
                        return 1;
                    }
                }
            }
        }
        _ => run_command_line(command)
    }
}

fn run_command_line(command: Vec<String>) -> i8 {
    let command_name = &command[0];
    let command_args: Vec<CString> = command.iter().map(|ref s| CString::new(s.as_bytes()).unwrap()).collect();

    if command_name.contains('/') {
        if Path::new(&command_name).exists() {
            return fork_exec(&CString::new(command_name.as_bytes()).unwrap(), &command_args);
        } else {
            println!("Could not file command {}", command_name);
            return 1;
        }
    } else {
        match get_path_for_command(command_name) {
            Some(command_path) => { return fork_exec(&command_path, &command_args); }
            None => {
                println!("Could not find command {}", command_name);
                return 1;
            }
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

fn fork_exec(command_name: &CString, command_args: &[CString]) -> i8 {
    use nix::unistd::{fork, ForkResult};
    use nix::sys::wait::WaitStatus;

    match fork() {
        Ok(ForkResult::Parent {child, ..}) => {
            return match nix::sys::wait::waitpid(child, None).expect("waitpid failed") {
                WaitStatus::Exited(_, status) => status,
                WaitStatus::Signaled(_, signal, core_dump_p) => -1,
                WaitStatus::Stopped(_, signal) => -2,
                _ => -3
            };
        }
        Ok(ForkResult::Child) => {
            nix::unistd::execv(command_name, command_args).expect("exec failed");
            return 0; // never reaches here, but compiler needs it
        }
        Err(_) => {
            println!("fork failed");
            return 1;
        }
    }
}
