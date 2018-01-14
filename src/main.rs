use std::env;
use std::io;
use std::io::{Write};
use std::ffi::{CString};
use std::path::{Path, PathBuf};

extern crate nix;
extern crate libc;

fn main() {
    let exit_code = real_main();
    std::process::exit(exit_code);
}

fn real_main() -> i32 {
    let mut command_parser = CommandParser::new();

    loop {
        match command_parser.get_next_command_list() {
            Some(command_list) => execute_command_list(command_list),
            None => break
        };
    }

    return 0;
}

#[derive(PartialEq,Debug)]
enum RunConditions {
    Always,
    IfTrue,
    IfFalse
}

#[derive(Debug)]
struct SimpleCommand {
    arguments: Vec<String>,
    run_conditions: RunConditions
}

impl SimpleCommand {
    fn always() -> SimpleCommand {
        return SimpleCommand {
            arguments: Vec::new(),
            run_conditions: RunConditions::Always
        }
    }

    fn if_true() -> SimpleCommand {
        return SimpleCommand {
            arguments: Vec::new(),
            run_conditions: RunConditions::IfTrue
        }
    }

    fn if_false() -> SimpleCommand {
        return SimpleCommand {
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

#[derive(Debug)]
enum Command {
    Simple(SimpleCommand),
    Subshell(CommandList)
}

//type CommandList = Vec<SimpleCommand>;
type CommandList = Vec<Command>;

struct CommandParser {
    token_index: usize,
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
            token_index: 0,
            command_list_buffer: Vec::new()
        }
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
        let mut tokens:Vec<CommandLineToken> = Vec::new();
        tokens = tokenize_command();
        if tokens.len() == 0 {
            return ParserStatus::EOF;
        }
        self.parse_command_line(tokens);
        return ParserStatus::Ok;
    }

    fn parse_command_line(&mut self, tokens: Vec<CommandLineToken>) {
        self.token_index = 0;
        let command_list = self.parse_command_list(&tokens);
        //println!("{:?}", command_list);
        self.command_list_buffer.push(command_list);
    }

    fn parse_command_list(&mut self, tokens: &Vec<CommandLineToken>) -> CommandList {
        let mut command_list: CommandList = Vec::new();
        let mut command: SimpleCommand = SimpleCommand::always();

        while self.token_index < tokens.len() {
            let token = &tokens[self.token_index];
            if token.class == CommandLineTokenType::Argument {
                self.parse_command(tokens, &mut command);
                command_list.push(Command::Simple(command));
                command = SimpleCommand::always();
                continue;
            } else if token.class == CommandLineTokenType::Semicolon {
                command = SimpleCommand::always();
            } else if token.class == CommandLineTokenType::EOL {
                return command_list;
            } else if token.class == CommandLineTokenType::AndOp {
                command = SimpleCommand::if_true();
            } else if token.class == CommandLineTokenType::OrOp {
                command = SimpleCommand::if_false();
            } else if token.class == CommandLineTokenType::OpenParen {
                self.token_index += 1;
                let subshell_command_list = self.parse_subshell(tokens);
                //println!("subshell command {:?}", subshell_command_list);
                command_list.push(Command::Subshell(subshell_command_list));
            } else if token.class == CommandLineTokenType::CloseParen {
                //parse error, but only if not in subshell
                return command_list;
            }

            self.token_index += 1;
        }

        return command_list;
    }

    fn parse_subshell(&mut self, tokens: &Vec<CommandLineToken>) -> CommandList {
        return self.parse_command_list(tokens);
    }

    fn parse_command(&mut self, tokens: &Vec<CommandLineToken>, command: &mut SimpleCommand) {
        while self.token_index < tokens.len() {
            let token = &tokens[self.token_index];
            if token.class == CommandLineTokenType::Argument {
                command.push_argument(token.lexeme.to_string());
            } else {
                return;
            }
            self.token_index += 1;
        }
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
    Semicolon,
    OpenParen,
    CloseParen
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

    fn open_paren() -> CommandLineToken {
        return CommandLineToken {
            class: CommandLineTokenType::OpenParen,
            lexeme: String::from("(")
        }
    }

    fn close_paren() -> CommandLineToken {
        return CommandLineToken {
            class: CommandLineTokenType::CloseParen,
            lexeme: String::from(")")
        }
    }
}

fn tokenize_command() -> Vec<CommandLineToken> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();
    let mut cur_arg_buf = String::new();
    let mut tokens = Vec::new();
    let mut line: std::string::String = String::new();

    stdout.write(b"$ ").unwrap();
    stdout.flush().unwrap();

    loop {
        let mut chars = match stdin.read_line(&mut line) {
            Ok(0) => return tokens,
            _ => line.chars()
        };

        loop {
            let c = match chars.next() {
                None => break,
                Some(ch) => ch
            };

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
                    tokens.push(CommandLineToken::eol());
                    return tokens;
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
            } else if c == '(' {
                if cur_arg_buf.len() > 0 {
                    tokens.push(CommandLineToken::argument(cur_arg_buf));
                    cur_arg_buf = String::new();
                }
                tokens.push(CommandLineToken::open_paren());
            } else if c == ')' {
                if cur_arg_buf.len() > 0 {
                tokens.push(CommandLineToken::argument(cur_arg_buf));
                    cur_arg_buf = String::new();
                }
                tokens.push(CommandLineToken::close_paren());
            } else {
                cur_arg_buf.push(c);
            }
        }

        stdout.write(b"> ").unwrap();
        stdout.flush().unwrap();
    }
}

fn execute_command_list(command_list: CommandList) -> i8 {
    use nix::unistd::{fork, ForkResult};
    use nix::sys::wait::WaitStatus;

    //println!("{:?}", command_list);
    let mut status = 0;
    for command in command_list {
        //println!("{:?}", command);
        match command {
            Command::Simple(sc) => {
                match sc.run_conditions {
                    RunConditions::Always => {
                        //println!("Always");
                        status = execute_command(sc.arguments)
                    },
                    RunConditions::IfTrue => {
                        //println!("IfTrue");
                        if status == 0 {
                            status = execute_command(sc.arguments)
                        }
                    },
                    RunConditions::IfFalse => {
                        //println!("IfFalse");
                        if status != 0 {
                            status = execute_command(sc.arguments)
                        }
                    }
                }
            }
            Command::Subshell(cl) => {
//              fork the process
//              wait in parent on child
//              in child, execute the command list, and exit with last executed status
                match fork() {
                    Ok(ForkResult::Parent {child, ..}) => {
                        match nix::sys::wait::waitpid(child, None).expect("waitpid failed") {
                            WaitStatus::Exited(_, s) => status = s,
                            WaitStatus::Signaled(_, _, _) => status = -1,
                            WaitStatus::Stopped(_, _) => status = -2,
                            _ => status = -3
                        };
                    }
                    Ok(ForkResult::Child) => {
                        status = execute_command_list(cl);
                        std::process::exit(status as i32);
                    }
                    Err(_) => {
                        println!("fork failed");
                        return 1;
                    }
                }
            }
        }

        //println!("command status: {}", status);
    }

    return status;
}

fn standardize_path(path: &Path) -> PathBuf{
    let mut standardized_path = PathBuf::new();
    if path.is_relative() {
        match env::var("PWD") {
            Ok(pwd) => standardized_path = PathBuf::from(pwd),
            Err(_) => standardized_path = PathBuf::from(env::current_dir().expect(""))
        }
    }

    for component in path.iter() {
        //println!("{:?}", component);
        match component.to_str().expect("") {
            "." => (),
            ".." => { standardized_path.pop(); },
            _ => standardized_path.push(component)
        }
    }

    return standardized_path;
}

fn execute_command(mut command: Vec<String>) -> i8 {
    if command.len() == 0 {
        return 0;
    }
    match &command[0][..] {
        "cd" => {
            if command.len() > 1 {
                let new_pwd =  standardize_path(Path::new(&command[1]));
                env::set_current_dir(&new_pwd).expect(""); // need to set command status on error here, so as not to panic
                env::set_var("PWD", &new_pwd);
                //println!("{:?} {:?}", env::current_dir(), env::var("PWD"));
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
        "exit" => {
            let status: i32;
            if command.len() > 1 {
                status = command[1].parse().unwrap();
            } else {
                status = 0; // should be the status of the last command, but thpbpbpbpt
            }
            std::process::exit(status);
        }
        "!" => {
            command.remove(0);
            let return_code = execute_command(command);
            if return_code != 0 { 0 } else { 1 }
        }
        "exec" => {
            command.remove(0);
            run_command_line(command, false)
        }
        _ => run_command_line(command, true)
    }
}

fn run_command_line(command: Vec<String>, fork: bool) -> i8 {
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
            Some(command_path) =>
                if fork {
                    return fork_exec(&command_path, &command_args);
                } else {
                    nix::unistd::execv(&command_path, &command_args).expect("exec failed");
                    return 0;
                }
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
