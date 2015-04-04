extern crate getopts;


use std::thread;
use getopts::{optopt, getopts};
use std::old_io::BufferedReader;
use std::old_io::stdin;
use std::{old_io, os};
use std::str;
use std::sync::mpsc;
use std::sync::mpsc::{channel};
use std::error::Error;
use std::io::prelude::*;
use std::process::{Command, Stdio};
fn main() {
    let opt_cmd_line = get_cmdline_from_args();

    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line.as_slice()),
        None           => Shell::new("gash > ").run(),
    }
}


fn old_main() {
    let input = "cat hello|grep moo";
    let x = GashCommandLine::new(input.as_slice());
    match x {
        GashCommandLine::Foreground(v) => {
            for a in v{
                a.spawn();
            }
        }
        _ => {}
    }
}

/// Gash command line is the main unit containing a line of commands. It is represented
/// here as a Vector to GashCommands
enum GashCommandLine<'a> {
    /// A foreground command is a standard command to run
    Foreground(Vec<GashCommand<'a>>),
    /// A background command is a line of commands ending with an &
    Background(Vec<GashCommand<'a>>),
    /// Empty command is an empty line
    Empty,
    /// Unsupported command (like && or ||)
    UnsupportedCommand,
    /// Exit is a command which starts with 'exit'
    Exit
}

/// Implements GashCommandLine
impl<'a> GashCommandLine<'a> {
    /// Constructor for a GashCommandLine
    fn new(line : & 'a str) -> GashCommandLine<'a> {
        // If the line is empty, this is an empty command
        if line.is_empty() {
            return GashCommandLine::Empty;
        }
        // If the first word is exit, this is an exit command
        if line.words().next().unwrap() == "exit" {
            return GashCommandLine::Exit;
        }
        if line.contains("||") || line.contains("&&"){
            return GashCommandLine::UnsupportedCommand;
        }
        let mut commands = Vec::new();
        // Split and parse each command as a new GashCommand
        for command_str in line.split('|'){
            commands.push(GashCommand::new(command_str));
        }
        // If this command ends with an & (potential bug - see '&&) make it a background
        // command. Otherwise - Foreground.
        match line.chars().last().unwrap(){
            '&' => GashCommandLine::Background(commands),
            _   => GashCommandLine::Foreground(commands),           
        }
    }
}

/// A gash command is a single command, separated from other commands by '|'.
enum GashCommand<'a> {
    /// A normal command is a command which can have STDIN, and has STDOUT and STDERR. Just
    /// like input and output redirect, it contains a Gash operation to execute.
    Normal(GashOperation<'a>),
    /// A history command is "meta", in that it refers to old commands
    History,
    /// A cd command changes the wd for the shell, its only content is a string containing the
    /// path of the directory to change to
    ChangeDirectory(Box<& 'a str>),
    /// Input redirect contains a GashOperation and a string, file directory to redirect input to
    InputRedirect(GashOperation<'a>, Box<& 'a str>),
    /// Output redirect - see input redirect.
    OutputRedirect(GashOperation<'a>, Box<& 'a str>),
    /// A bad command, due to bad parsing.
    BadCommand,
}


/// A gash command implementation
impl<'a> GashCommand<'a> {
    /// Constructor for GashCommand, takes in the wording of the command
    fn new(full_command : & 'a str) -> GashCommand<'a> {
        // full_command includes possible redirection
        // Separates command into tokens on white space
        let mut full_command_words = full_command.words();
        // Operator - first token
        let operator = full_command_words.next().unwrap();
        // Matches on operator, dispatches GashCommand
        match operator {
            "cd" => return GashCommand::ChangeDirectory(Box::new(full_command_words.next().unwrap())),

            "history" =>        return GashCommand::History,

            // Output redirect, splits further to get location of directory
            _   if full_command.contains(">") => {
                let mut command = full_command.split_str(">");
                let mut tokens = command.next().unwrap().words();
                let operator = tokens.next().unwrap();
                GashCommand::OutputRedirect( 
                    GashOperation{ operator:Box::new(operator),
                        operands:Box::new(tokens.collect()) },
                    Box::new(command.next().unwrap()) );
            }

            // Input redirect, same as above
            _   if full_command.contains("<") => {
                let mut command = full_command.split_str("<");
                let mut tokens = command.next().unwrap().words();
                let operator = tokens.next().unwrap();
                GashCommand::InputRedirect(
                    GashOperation{ operator:Box::new(operator),
                        operands:Box::new(tokens.collect()) },
                    Box::new(command.next().unwrap()) );
            }

            //Otherwise, this is just a normal command
            _   =>  return GashCommand::Normal(
                GashOperation{ operator:Box::new(operator),operands:Box::new(full_command_words.collect()) } ),
        }

        //If match doesn't get executed, we still need to return a command. Hence - bad command.
        GashCommand::BadCommand
    }

    //Testing: ignore.
    fn spawn(&self){
        thread::scoped(move || {println!("this is thread number ");});
    }
}


///A GashOperation is the basic unit of an operation, contains an operator ("echo") and a vector of
///operands (arguments to operator).
struct GashOperation<'a>{
    operator : Box<& 'a str>,
    operands: Box<Vec<& 'a str>>
}












struct Shell<'a> {
    cmd_prompt: &'a str,
}

impl <'a>Shell<'a> {
    fn new(prompt_str: &'a str) -> Shell<'a> {
        Shell { cmd_prompt: prompt_str }
    }

    fn run(&self) {
        let mut stdin = BufferedReader::new(stdin());
        let mut history: Vec<String> = Vec::new();
        loop {
            old_io::stdio::print(self.cmd_prompt.as_slice());
            old_io::stdio::flush();
            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim();
            let command = GashCommandLine::new(cmd_line);
            match command {
                GashCommandLine::Foreground(cmds) => self.run_cmds(cmds),
                GashCommandLine::Background(cmds) => println!("back task!"),
                GashCommandLine::Exit             => return,
                _                                 => println!("other")
            }

            history.push(String::from_str(cmd_line));
        }
    }

    fn run_cmds(&self, cmds: Vec<GashCommand>){
        let mut rx_stack= Vec::new();
        let mut tx_stack = Vec::new();

        // Initialize a Vec full of channels
        tx_stack.push(None);
        for _ in 0..(cmds.len() - 1) {
            let (tx, rx) = channel::<String>();
            rx_stack.push(Some(rx));
            tx_stack.push(Some(tx));
        }
        rx_stack.push(None);
        for cmd in cmds { 
            match cmd {
                GashCommand::Normal(op) => {
                    let output = Command::new(*op.operator).args(&*op.operands.as_slice()).output().unwrap_or_else(|e| {panic!("failed to execute process: {}", e)});
                    let stderr=String::from_utf8_lossy(&output.stderr);
                    let stdout=String::from_utf8_lossy(&output.stdout);
                    if !"".eq(stdout.as_slice()) {
                        print!("{}", stdout);
                    }
                    if !"".eq(stderr.as_slice()) {
                        print!("{}", stderr);
                    }

                },
                _ => println!("shit happens")
            };
        }
    }

    fn run_cmdline(&self, cmd_line: &str) {
        let argv: Vec<&str> = cmd_line.split(' ').filter_map(|x| {
            if x == "" {
                None
            } else {
                Some(x)
            }
        }).collect();

        match argv.first() {
            Some(&program) => self.run_cmd(program, argv.tail()),
            None => (),
        };
    }

    fn run_cmd(&self, program: &str, argv: &[&str]) {
        if self.cmd_exists(program) {
            let output = Command::new(program).args(argv).output().unwrap_or_else(|e| {panic!("failed to execute process: {}", e)});
            let stderr=String::from_utf8_lossy(&output.stderr);
            let stdout=String::from_utf8_lossy(&output.stdout);
            if !"".eq(stdout.as_slice()) {
                print!("{}", stdout);
            }
            if !"".eq(stderr.as_slice()) {
                print!("{}", stderr);
            }
        } else {
            println!("{}: command not found", program);
        }
    }

    fn cmd_exists(&self, cmd_path: &str) -> bool {
        Command::new("which").arg(cmd_path).stdout(Stdio::capture()).status().unwrap().success()
    }
}

fn get_cmdline_from_args() -> Option<String> {
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();

    let opts = &[
    getopts::optopt("c", "", "", "")
    ];

    getopts::getopts(args.tail(), opts).unwrap().opt_str("c")
}


