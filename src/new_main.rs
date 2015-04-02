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
use std::sync::mpsc;

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

        // Top level of REPL loop
        loop {
            // Vars for this command string
            let thread_stack = Vec::new();  // Holds JoinHandles for spawned threads

            // Flush stdio and read a new command string
            old_io::stdio::print(self.cmd_prompt.as_slice());
            old_io::stdio::flush();
            let line = stdin.read_line().unwrap();
            
            // PARSE CODE GOES HERE
            let cmd_struct = PARSE PARSE PARSE

            // Used later to drop or keep join handles
            let background? = cmd_struct.background;

            // For sizing channel Vecs
            let num_threads = cmd_struct.thread_count;

            // Initialize and populate channel Vecs
            let sender_stack = Vec::new();
            let receiver_stack = Vec::new();
            sender_stack.push(None);
            for _ in 0..(num_threads - 1) {
                let (tx, rx) = channel::<String>();
                receiver_stack.push(Some(rx));
                sender_stack.push(Some(tx));
            }
            receiver_stack.push(None);

            // Ready to start spawning threads

            // Iterate through the parsed structs and spawn a super thread for each
            for cmd_struct in placeholder_iter {

                // Decide which type of thread to spawn
                // Spawn master thread, returns join handle, pass in channel handles
                    // Spawn command (if appropriate)
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

                    // Spawn helper threads

                    // Terminate when eof read on stdout


                // Completed with this thread, iterate to next one

            }

            //Completed with all threads, if flag set drop handles, else join
            if !background? {for thread in thread_stack.iter() { thread.join(); } }

            
            history.push(String::from_str(cmd_line));
        }
    }

    // Runs command with args
    // Validates by calling cmd_exists() first
    // Returns handle to the Command after spawning it
    fn run_cmd(&self, program: &str, args: &[&str]) -> Result<Child>{
        if self.cmd_exists(program) {
        Command::new(program).args(args)
                .stdin(process::Stdio::capture()).stdout(process::Stdio::capture())
                .stderr(process::Stdio::capture()).spawn()
        } else {
            Err("Command not found")
        }
    }


    // REMOVING THIS BECAUSE IT HANDLES BEHAVIOR HANDLED ELSEWHERE
    // fn run_cmd(&self, program: &str, argv: &[&str]) {
    //     if self.cmd_exists(program) {
    //         let output = Command::new(program).args(argv).output().unwrap_or_else(|e| {panic!("failed to execute process: {}", e)});
    //         let stderr=String::from_utf8_lossy(&output.stderr);
    //         let stdout=String::from_utf8_lossy(&output.stdout);
    //         if !"".eq(stdout.as_slice()) {
    //             print!("{}", stdout);
    //         }
    //         if !"".eq(stderr.as_slice()) {
    //             print!("{}", stderr);
    //         }
    //     } else {
    //         println!("{}: command not found", program);
    //     }
    // }

    // Uses a 'which' command on underlying system to validate command before execution
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

