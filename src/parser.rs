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


enum GashCommandLine<'a> {
    Foreground(Vec<GashCommand<'a>>),
    Background(Vec<GashCommand<'a>>),
    Empty,
    Exit
}

impl<'a> GashCommandLine<'a> {
    fn new(line : & 'a str) -> GashCommandLine<'a> {
        if line.is_empty() {
            return GashCommandLine::Empty;
        }
        if line.words().next().unwrap() == "exit" {
            return GashCommandLine::Exit;
        }
        let mut commands = Vec::new();
        for command_str in line.split('|'){
            commands.push(GashCommand::new(command_str));
        }
        match line.chars().last().unwrap(){
            '&' => GashCommandLine::Background(commands),
            _   => GashCommandLine::Foreground(commands),           
        }
    }
}

impl<'a> GashCommand<'a> {
    fn new(command : & 'a str) -> GashCommand<'a> {
        let mut tokens = command.words();
        let operator = tokens.next().unwrap();
        match operator {
            "cd" => return GashCommand::ChangeDirectory(Box::new(command.words().next().unwrap())),

            "history" =>        return GashCommand::History,

            _   if command.contains(">") => {
                let mut command = command.split_str(">");
                let mut tokens = command.next().unwrap().words();
                let operator = tokens.next().unwrap();
                GashCommand::OutputRedirect(GashOperation{operator:Box::new(operator), operands:Box::new(tokens.collect())}, Box::new(command.next().unwrap()));},
            _   if command.contains(">") => {
                let mut command = command.split_str(">");
                let mut tokens = command.next().unwrap().words();
                let operator = tokens.next().unwrap();
               GashCommand::InputRedirect(GashOperation{operator:Box::new(operator), operands:Box::new(tokens.collect())}, Box::new(command.next().unwrap()));

                },

                _   =>  return GashCommand::Normal(GashOperation{operator:Box::new(operator),operands:Box::new(tokens.collect())}),
        }
        GashCommand::BadCommand
    }

    fn spawn(&self){
        thread::scoped(move || {println!("this is thread number ");});
    }
}



struct GashOperation<'a>{
    operator : Box<& 'a str>,
    operands: Box<Vec<& 'a str>>
}



enum GashCommand<'a> {
    Normal(GashOperation<'a>),
    History,
    ChangeDirectory(Box<& 'a str>),
    InputRedirect(GashOperation<'a>, Box<& 'a str>),
    OutputRedirect(GashOperation<'a>, Box<& 'a str>),
    BadCommand,
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
        for cmd in cmds {
            let (tx, rx) = channel::<String>();
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


