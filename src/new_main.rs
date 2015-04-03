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
            // TODO: Add Parse code
            let cmd_struct = PARSE PARSE PARSE

            // Used later to drop or keep join handles
            // TODO: Match this to parse structure
            let background? = cmd_struct.background;

            // For sizing channel Vecs
            // TODO: Get this from parse structure
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

            // Ready to start spawning threads, check for special cases
            match program {
                ""      =>  { continue; }
                "exit"  =>  { return; }
                "cd"    => {    // TODO: GET ARGS FOR CD
                    match cmd_line.splitn(1, ' ').nth(1) { 
                        None => {os::change_dir(&os::homedir().unwrap());}
                        Some(path) => {os::change_dir(&Path::new(path)); continue;}
                    }
                }
                _       =>  {}  // Do nothing. All other branches end the loop.
            };

            // Iterate through the parsed structs and spawn a super thread for each
            for cmd_struct in placeholder_iter {
                // Pop local references to channels
                let rx = receiver_stack.pop().unwrap();
                let tx = sender_stack.pop().unwrap();

                // Decide which type of thread to spawn (history is special case)
                match program {
                    "history" => { // TODO: Put History Thread here, feed pipe or print }
                    _ => { // Spawn master thread, returns join handle, 

                        // Spawn command
                        let process_handle = Some( self.run_cmd(cmd_struct) )

                        // Spawn helper threads
                        let in_helper = match rx {
                            Some(receiver) => {// Spawn a thread to handle in pipe
                                let stdin = process_handle.stdin;
                                thread::scoped(move || {
                                    // Feed process from input channel until channel closes
                                    loop {
                                        let write_result = match receiver.recv() {
                                            Ok(msg) => stdin.write_all(msg.as_bytes());
                                            Err(_) => { break }
                                        }
                                        match write_result {
                                            Ok(_) => { continue; }
                                            Err(_) => { println!("Error: Failed writing to channel"); break; }
                                        }
                                    }
                                })
                            }
                            None => { let a = process_handle.stdin; None } // No in pipe, just drop handle
                        }
                        let out_helper = match tx {
                            Some(sender) => {// Spawn a thread to pass on out pipe
                                let stdout = process_handle.stdout;
                                thread::scoped(move || {
                                    let process_reader = StdOutIter{ out : stdout };

                                    for output in process_reader {
                                        sender.send(output)
                                    }
                                })
                            }
                            None => { // Spawn a thread to print from out pipe
                                let stdout = process_handle.stdout;
                                thread::scoped(move || {
                                    let process_reader = StdOutIter{ out : stdout };

                                    for output in process_reader {
                                        print!("{}", output);
                                    }
                                })

                            }
                        }

                        // Helper thread handles drop, joining on them.

                    }  // End of non-history program handling
                    
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

// Struct to encapsulate iteration over stdout from a spawned process
// Calling next reads the next buffer length, chops it to size,
// or returns None when the pipe is done.
const BUFFER_SIZE :usize = 80;
struct StdOutIter {
    out: process::ChildStdout,
}
impl<'a> Iterator for StdOutIter {
    type Item = String;

    fn next(& mut self) -> Option<String> {
        let mut buffer_array : [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let buffer = &mut buffer_array;
        
        let output_str = match self.out.read(buffer) {
            Ok(length) => if length == 0 { return None }
                            else { str::from_utf8(&buffer[0..length]) },
            Err(_)   => { return None },
        };

        match output_str {
            Ok(string) => Some(string.to_string()),
            Err(_) => panic!("failed to convert stdin to String"),
        }

    }
}

fn main() {
    let opt_cmd_line = get_cmdline_from_args();

    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line.as_slice()),
        None           => Shell::new("gash > ").run(),
    }
}

