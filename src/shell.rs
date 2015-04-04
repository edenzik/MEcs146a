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
mod gash;

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
