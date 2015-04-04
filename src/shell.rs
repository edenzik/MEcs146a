extern crate getopts;

use std::thread;
use getopts::{optopt, getopts};
use std::old_io::BufferedReader;
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
        let mut stdin = BufferedReader::new(old_io::stdin());
        let mut history: Vec<String> = Vec::new();

        // Main REPL loop, may spawn background jobs to finish
        loop {
            // Get command string from user
            old_io::stdio::print(self.cmd_prompt.as_slice());
            old_io::stdio::flush();

            // Try to read from stdin
            // If successful, create a GashCommandLine, otherwise let user try again
            let command_string = match stdin.read_line() {
                Ok(input_line) => input_line,
                Err(msg) => { println!("Failed to read from stdin: {}", msg); continue; }
            };

            let gash_command_line = 
                gash::GashCommandLine::new(command_string, history.clone());

            // Branch depending on parse of input
            match gash_command_line {
                // Special cases:
                gash::GashCommandLine::Empty => { continue; }  // Present another prompt
                gash::GashCommandLine::Exit => { break; }      // End REPL loop
                
                // Invalid input
                gash::GashCommandLine::UnsupportedCommand(msg) => { println!("{}", msg); continue; }

                // Else, run this well-formed batch of commands
                _ => { gash_command_line.run_batch(); }
            };

            // Add this history to the record
            history.push( command_string );
        }
    }
}

// Create and start a new shell
fn main() {
    Shell::new("gash > ").run();
}