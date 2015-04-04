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
use std::process;


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
    fn new(input_line : & 'a str) -> GashCommandLine<'a> {
        if input_line.is_empty() {
            // If the line is empty, this is an empty command
            GashCommandLine::Empty
        } else if input_line.words().next().unwrap() == "exit" {
            // If the first word is exit, this is an exit command
            GashCommandLine::Exit
        } else if input_line.contains("||") || input_line.contains("&&"){
            // Multiple commands per line are not supported
            GashCommandLine::UnsupportedCommand
        } else {
            let mut gash_command_vec = Vec::new();
            for command_str in input_line.split('|'){
                gash_command_vec.push(GashCommand::new(command_str));
            }
            match input_line.chars().last().unwrap(){
                '&' => {
                    let temp_last = gash_command_vec.pop();
                    temp_last.pop();
                    gash_command_vec.push(temp_last);
                    GashCommandLine::Background(gash_command_vec)
                },
                _   => GashCommandLine::Foreground(gash_command_vec),           
            }
            // Else case: one or more subcommands piped together
            // Background/Foreground is handled by return type
            // Split and parse each command as a new GashCommand
            // If this command ends with an & (potential bug - see '&&) make it a background
            // command. Otherwise - Foreground.
        }
    }

    /* Example usage of run_batch:
       let gash_cmd_line = GashCommandLine::new(input_line);
       match gash_cmd_line {
       Empty => { continue; }
       Exit => { break; }
       UnsupportedCommand(msg) => { println!("{}", msg); continue; }
       _ => { gash_cmd_line.run_batch(); }
       };

*/

    fn run_batch(&self) {

        // Initialize and populate channel Vecs
        let sender_stack = Vec::new();
        let receiver_stack = Vec::new();
        sender_stack.push(None);
        for _ in 0..(5 - 1) {
            let (tx, rx) = channel::<String>();
            receiver_stack.push(Some(rx));
            sender_stack.push(Some(tx));
        }
        receiver_stack.push(None);

        match *self {
            GashCommandLine::Background(command_vec) => {
                // Spawn each as an unscoped thread, let handles drop
                for gash_command in command_vec.iter() {
                    // Get channel handles
                    let tx = sender_stack.pop().unwrap();
                    let rx = receiver_stack.pop().unwrap();
                    gash_command.run(tx, rx).spawn().unwrap();
                }
            }
            GashCommandLine::Foreground(command_vec) => {
                // Spawn each as a scoped thread. Drop handles.
                let mut handles = Vec::new();
                for gash_command in command_vec.iter() {
                    // Get channel handles
                    let tx = sender_stack.pop().unwrap();
                    let rx = receiver_stack.pop().unwrap();
                    handles.push( gash_command.run(tx, rx).scoped().unwrap() );
                }
            }
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
            "cd" => GashCommand::ChangeDirectory(
                Box::new( full_command_words.next().unwrap() ) ),

                "history" => GashCommand::History,

                _   if !self.cmd_exists(operator) => GashCommand::UnsupportedCommand,

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
                    Box::new(command.next().unwrap()) )
            }

            // Otherwise, this is just a normal command
            _   =>  GashCommand::Normal(
                GashOperation{ operator:Box::new(operator),
                operands:Box::new(full_command_words.collect()) } ),
        }
        /* This should go inside the match as part of the valid check

        // If match doesn't get executed, we still need to return a command. 
        // Hence - bad command.
        GashCommand::BadCommand
        */
    }

    fn cmd_exists(&self, cmd_path: &str) -> bool {
        Command::new("which").arg(cmd_path).stdout(Stdio::capture()).status().unwrap().success()
    }

    fn get_cmdline_from_args() -> Option<String> {
        /* Begin processing program arguments and initiate the parameters. */
        let args = os::args();

        let opts = &[
            getopts::optopt("c", "", "", "")
            ];

        getopts::getopts(args.tail(), opts).unwrap().opt_str("c")
    }

    /// running a GashCommand starts a thread and returns a JoinHandle to that thread
    /// accepts Sender and Receiver channels (or None) for piping
    /// matches on variant of GashCommand to determine thread's internal behavior
    fn run(&self, thread_tx : mpsc::Sender, thread_rx : mpsc::Receiver) -> thread::JoinHandle {
        match *self {
            // Standard form, make process and helper threads to connect pipes and channels
            GashCommand::Normal(gash_operation) => { thread::spawn( move || {

                // Spawn command as a process
                let process_handle = gash_operation.run_cmd();

                // Spawn helper threads
                let in_helper = match thread_rx {
                    Some(receiver) => {// Spawn a thread to handle in pipe
                        let stdin = process_handle.stdin;
                        thread::scoped(move || {
                            // Feed process from input channel until channel closes
                            loop {
                                let write_result = match receiver.recv() {
                                    Ok(msg) => stdin.write_all(msg.as_bytes()),
                                    Err(_) => { break }
                                };
                                match write_result {
                                    Ok(_) => { continue; }
                                    Err(_) => { println!("Error: Failed writing to channel"); break; }
                                }
                            }
                        })
                    }
                    None => { let a = process_handle.stdin; None } // No in pipe, just drop handle
                };
                let out_helper = match thread_tx {
                    Some(sender) => {// Spawn a thread to pass on out pipe
                        let stdout = process_handle.stdout;
                        thread::scoped(move || {
                            let process_reader = StdOutIter{ out : stdout };

                            for output in process_reader {
                                sender.send(output)
                            }
                        })
                    },
                    None => { // Spawn a thread to print from out pipe
                        let stdout = process_handle.stdout;
                        thread::scoped(move || {
                            let process_reader = StdOutIter{ out : stdout };

                            for output in process_reader {
                                print!("{}", output);
                            }
                        });

                    },
                };

                // Helper thread handles drop, joining on them.

            })}

            // No process--use thread to read history
            GashCommand::History => {}

            // If tx and rx are None, change system directory. Else do nothing.
            // This is the observed behavior from testing on Ubuntu 14.04
            GashCommand::ChangeDirectory( file_name ) => {
                match (thread_tx, thread_rx) {
                    // If both none, actually change the directory
                    (None, None) => { thread::spawn(move || {
                        match *file_name { 
                            None => {os::change_dir(&os::homedir().unwrap());}
                            Some(path) => {os::change_dir(&Path::new(path)); }
                        }
                    }).ok() }
                    _ => { thread::spawn(move || {} ).ok() } // Do nothing
                }
            }

            // Similar to Normal, but have input helper thread feed from file instead of channel
            GashCommand::InputRedirect( gash_operation, file_name ) => {}

            // Similar to Normal, but have output helper thread feed to file instead of channel
            GashCommand::OutputRedirect( gash_operation, file_name ) => {}

            // GashCommandLine should not allow running a line that has a bad command in it
            GashCommand::BadCommand => { panic!("ERROR: Attempted to run BadCommand") }
        }
    }

}   // End of impl GashCommand


/// A GashOperation is the basic unit of an operation, contains an operator ("echo") 
/// and a vector of operands (arguments to operator).
struct GashOperation<'a> {
    operator : Box<& 'a str>,
    operands: Box<Vec<& 'a str>>
}

impl<'a> GashOperation<'a> {
    // Runs command with args
    // Returns handle to the Command after spawning it
    fn run_cmd(&self) -> Result<Child>{
        Command::new(*self.operator).args(&*self.operands.as_slice())
            .stdin(process::Stdio::capture()).stdout(process::Stdio::capture())
            .stderr(process::Stdio::capture()).spawn()
    }
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

