//
// gash.rs
//
// Starting code for PA2
// Running on Rust 1.0.0 - build 02-21
//
// Brandeis University - cs146a - Spring 2015

extern crate getopts;

use getopts::{optopt, getopts};
use std::old_io::BufferedReader;
use std::process::{Command, Stdio};
use std::old_io::stdin;
use std::{old_io, os};
use std::str;

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
            let program = cmd_line.splitn(1, ' ').nth(0).expect("no program");

            match program {
                ""      =>  { continue; }
                "exit"  =>  { return; }
                "history" => {println!("{:?}",history);}
                "cd"    => {
                     match cmd_line.splitn(1, ' ').nth(1) {
                        None => {os::change_dir(&os::homedir().unwrap());}
                        Some(path) => {os::change_dir(&Path::new(path));}
                     }; 
                 }
                _       =>  { self.run_cmdline(cmd_line); }
            }
            history.push(String::from_str(cmd_line));
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

fn main() {
    let opt_cmd_line = get_cmdline_from_args();

    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line.as_slice()),
        None           => Shell::new("gash > ").run(),
    }
}

