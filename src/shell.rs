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

    // Begins the REPL loop
    fn run(&self) {
        let mut stdin = BufferedReader::new(stdin());
        let mut history: Vec<String> = Vec::new();

        // Main REPL loop, may spawn background jobs to finish
        loop {
            // Get command string from user
            old_io::stdio::print(self.cmd_prompt.as_slice());
            old_io::stdio::flush();

            // Try to read from stdin
            // If successful, create a GashCommandLine, otherwise let user try again
            let gash_command_line = match stdin.read_line() {
                Ok(input_line) => GashCommandLine::new(input_line, history.clone()),
                Err(msg) => { println!("Failed to read from stdin: {}", msg); continue; }
            };

            // Branch depending on parse of input
            match gash_cmd_line {
                // Special cases:
                Empty => { continue; }  // Present another prompt
                Exit => { break; }      // End REPL loop
                UnsupportedCommand(msg) => { println!("{}", msg); continue; } // Invalid input
                
                // Else, run this well-formed batch of commands
                _ => { gash_cmd_line.run_batch(); }
            };

            // Add this history to the record
            history.push(String::from_str(gash_command_line));
        }
    }
}

// Code supplied as part of initial setup. Used for executing a single command with gash.
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