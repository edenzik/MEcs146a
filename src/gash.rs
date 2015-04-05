use std::thread::{self, JoinHandle};
use std::{old_io, os};
use std::fs::File;
use std::str;
use std::sync::mpsc::{self, Sender, Receiver};
use std::process;
use std::io::{self, Read, Write, Result};
// use std::error::Error;

/// Gash command line is the main unit containing a line of commands. It is represented
/// here as a Vector to GashCommands
pub enum GashCommandLine<'a> {
    /// A foreground command is a standard command to run
    Foreground(Vec<GashCommand<'a>>),
    /// A background command is a line of commands ending with an &
    Background(Vec<GashCommand<'a>>),
    /// Empty command is an empty line
    Empty,
    /// Unsupported command (like && or ||)
    UnsupportedCommand(& 'a str),
    /// Exit is a command which starts with 'exit'
    Exit
}

/// Implements GashCommandLine
impl<'a> GashCommandLine<'a> {
    /// Constructor for a GashCommandLine
    pub fn new(input_line : & 'a str, history : Vec<String>) -> GashCommandLine<'a> {
        if input_line.is_empty() {
            // If the line is empty, this is an empty command
            GashCommandLine::Empty
        } else if input_line.words().next().unwrap() == "exit" {
            // If the first word is exit, this is an exit command
            GashCommandLine::Exit
        } else if input_line.contains("||") || input_line.contains("&&"){
            // Multiple commands per line are not supported
            GashCommandLine::UnsupportedCommand("we dont support || or &&.")
        } else {
            match input_line.chars().last().unwrap(){
                '&' => {
                    let removed_tip = input_line.slice_chars(0,input_line.len()-1);
                    let mut gash_command_vec = 
                        GashCommandLine::create_gash_commands(removed_tip, history);
                    return GashCommandLine::Background(gash_command_vec);
                },
                _   => {

                    let mut gash_command_vec =
                        GashCommandLine::create_gash_commands(input_line, history);
                    return GashCommandLine::Foreground(gash_command_vec);
                }
            }
            // Else case: one or more subcommands piped together
            // Background/Foreground is handled by return type
            // Split and parse each command as a new GashCommand
            // If this command ends with an & (potential bug - see '&&) make it a background
            // command. Otherwise - Foreground.
        }
    }

    fn create_gash_commands(input :  & 'a str, history : Vec<String>) -> Vec<GashCommand>{
        let mut gash_command_vec = Vec::new();
        for command_str in input.split('|'){
            gash_command_vec.push(GashCommand::new(command_str, history.clone()));
        }
        return gash_command_vec;
    }


    pub fn run_batch(self) {
        // Initialize and populate channel Vecs
        let mut sender_stack = Vec::new();
        let mut receiver_stack = Vec::new();
        sender_stack.push(None);

        match self {
            GashCommandLine::Background(command_vec) => {
                for _ in 0..(command_vec.len() - 1) {
                    let (tx, rx) = mpsc::channel::<String>();
                    receiver_stack.push(Some(rx));
                    sender_stack.push(Some(tx));
                }
                receiver_stack.push(None);

                // Spawn each as an unscoped thread, let handles drop
                for gash_command in command_vec.into_iter() {
                    // Get channel handles
                    let tx = sender_stack.pop().unwrap();
                    let rx = receiver_stack.pop().unwrap();
                    gash_command.run(tx, rx);
                }
                // Let handles drop (detach) and allow command to run in background
            }
            GashCommandLine::Foreground(command_vec) => {
                for _ in 0..(command_vec.len() - 1) {
                    let (tx, rx) = mpsc::channel::<String>();
                    receiver_stack.push(Some(rx));
                    sender_stack.push(Some(tx));
                }
                receiver_stack.push(None);

                // Spawn each as a scoped thread. Drop handles.
                let mut handles = Vec::new();
                for gash_command in command_vec.into_iter() {
                    // Get channel handles
                    let tx = sender_stack.pop().unwrap();
                    let rx = receiver_stack.pop().unwrap();
                    let handle = gash_command.run(tx,rx);
                    handles.push( handle );
                }
                // Join on all handles to force command to run in foreground
                for handle in handles.into_iter() {
                    handle.join();
                }
            }
            // Other matches covered in previous case
            _ => panic!("Error: attempted to start batch of commands--batch not well-formed.")
        }
    }
}

/// A gash command is a single command, separated from other commands by '|'.
enum GashCommand<'a> {
    /// A normal command is a command which can have STDIN, and has STDOUT and STDERR. Just
    /// like input and output redirect, it contains a Gash operation to execute.
    Normal(GashOperation),
    /// A history command is "meta", in that it refers to old commands
    History(Vec<String>),
    /// A cd command changes the wd for the shell, its only content is a string containing the
    /// path of the directory to change to
    ChangeDirectory(Box<& 'a str>),
    /// Input redirect contains a GashOperation and a string, file directory to redirect input to
    InputRedirect(GashOperation, Box<& 'a str>),
    /// Output redirect - see input redirect.
    OutputRedirect(GashOperation, Box<& 'a str>),
    //A command that is bad
    BadCommand(Box<& 'a str>)
}


/// A gash command implementation
impl<'a> GashCommand<'a> {
    /// Constructor for GashCommand, takes in the wording of the command
    fn new(full_command : & 'a str, history : Vec<String>) -> GashCommand<'a> {
        // full_command includes possible redirection
        // Separates command into tokens on white space
        let mut full_command_words = full_command.words();
        // Operator - first token
        let operator = full_command_words.next().unwrap();
        // Matches on operator, dispatches GashCommand
        match operator {
            "cd" => GashCommand::ChangeDirectory(
                Box::new( full_command_words.next().unwrap() ) ),

                "history" => GashCommand::History(history),

                _   if !GashCommand::cmd_exists(operator) => GashCommand::BadCommand(Box::new(operator)),

                // Output redirect, splits further to get location of directory
                _   if full_command.contains(">") => {
                    let mut command = full_command.split_str(">");
                    let mut tokens = command.next().unwrap().words();
                    let operator = tokens.next().unwrap();
                    GashCommand::OutputRedirect( 
                        GashOperation::new(Box::new(operator),Box::new(tokens.collect())),
                        Box::new(command.next().unwrap()) )
                }

            // Input redirect, same as above
            _   if full_command.contains("<") => {
                let mut command = full_command.split_str("<");
                let mut tokens = command.next().unwrap().words();
                let operator = tokens.next().unwrap();
                GashCommand::InputRedirect(
                    GashOperation::new(Box::new(operator),Box::new(tokens.collect())), Box::new(command.next().unwrap()))
            }

            // Otherwise, this is just a normal command
            _   =>  GashCommand::Normal(
                GashOperation::new(Box::new(operator), Box::new(full_command_words.collect()))),
        }
        /* This should go inside the match as part of the valid check

        // If match doesn't get executed, we still need to return a command. 
        // Hence - bad command.
        GashCommand::BadCommand
        */
    }

    fn cmd_exists(cmd_path: &str) -> bool {
        process::Command::new("which").arg(cmd_path).stdout(process::Stdio::capture())
            .status().unwrap().success()
    }

    /// running a GashCommand starts a thread and returns a JoinHandle to that thread
    /// accepts Sender and Receiver channels (or None) for piping
    /// matches on variant of GashCommand to determine thread's internal behavior
    fn run(self, thread_tx : Option<mpsc::Sender<String>>,
           thread_rx : Option<mpsc::Receiver<String>>) -> thread::JoinHandle {
               match self {
                   // Standard form, make process and helper threads to connect pipes and channels
                   GashCommand::Normal(gash_operation) => { GashCommand::start_piped_process(thread_tx,
                                                                                                        thread_rx, gash_operation) }

                   // No process--use thread to read history
                   GashCommand::History(_) => { panic!("Whoops, forgot to implement history!") }

                   // If tx and rx are None, change system directory. Else do nothing.
                   // This is the observed behavior from testing on Ubuntu 14.04
                   GashCommand::ChangeDirectory( file_name ) => {
                       match (thread_tx, thread_rx) {
                           // If both none, actually change the directory
                           (None, None) => { 
                               match *file_name { 
                                   "" => {
                                       os::change_dir(&os::homedir().unwrap()); }
                                   path => { 
                                       os::change_dir(&Path::new(path)).unwrap(); }
                               };
                               thread::spawn(move || { }) }
                           _ => { thread::spawn(move || {} ) } // Do nothing
                       }

                   }

                   // Similar to Normal, but add another thread to read from file and feed into thread
                   GashCommand::InputRedirect( gash_operation, file_name ) => { 
                       // Don't need input channel
                       drop(thread_rx);
                       let (file_sender, file_receiver) = mpsc::channel::<String>();

                       // Thread to read from file and write into newly created channel
                       GashCommand::create_io_reader(file_sender, file_name);

                       // Now start command like normal with new channel to read from
                       GashCommand::start_piped_process(thread_tx, Some(file_receiver), gash_operation)

                   }

                   // Similar to Normal, add another thread to read from thread and write into file
                   GashCommand::OutputRedirect( gash_operation, file_name ) => { 
                       // Don't need output channel
                       drop(thread_tx);
                       let (file_sender, file_receiver) = mpsc::channel::<String>();

                       // Start command like normal with new channel to write to,
                       // grabbing handle to return
                       let handle = GashCommand::start_piped_process(Some(file_sender), 
                                                                                                        thread_rx, gash_operation);

                       // Thread to write to file, reading from newly created channel
                       GashCommand::create_io_writer(file_receiver, file_name);

                       handle

                   }

                   // GashCommandLine should not allow running a line that has a bad command in it
                   GashCommand::BadCommand(operator) =>  {
                       let operator_name = String::from_str(*operator);
                       return thread::spawn(move||{println!("gash:\t command not found: {}", operator_name)});
                   }
                   
               }
           }

    // Starts process from GashOperation data, connects process' pipes to channels via threads,
    // and returns handle to overall thread for joining or dropping
    fn start_piped_process<'b>(tx_channel : Option<mpsc::Sender<String>>,
                               rx_channel : Option<mpsc::Receiver<String>>, gash_op : GashOperation)
        -> thread::JoinHandle {




            thread::spawn( move || {
                // Spawn command as a process
                let process_handle = gash_op.run_cmd().unwrap();

                // Spawn helper threads
                let in_helper = match rx_channel {
                    Some(receiver) => {// Spawn a thread to handle in pipe
                        let mut stdin = process_handle.stdin.unwrap();
                        Some( thread::scoped(move || {
                            // Feed process from input channel until channel closes
                            loop {
                                let write_result = match receiver.recv() {
                                    Ok(msg) => stdin.write_all(msg.as_bytes()),
                                    Err(_) => { break }
                                };
                                match write_result {
                                    Ok(_) => { continue; }
                                    Err(_) => { println!("Error: Failed writing to channel");
                                        break; }
                                }
                            }
                        }) )
                    }
                    None => { let a = process_handle.stdin; None } // No in-pipe, just drop handle
                };
                let out_helper = match tx_channel {
                    Some(sender) => {// Spawn a thread to pass on out pipe
                        let stdout = process_handle.stdout.unwrap();
                        thread::scoped(move || {
                            let process_reader = StdOutIter{ out : stdout };

                            for output in process_reader {
                                sender.send(output).unwrap();
                            }
                        })
                    }
                    None => { // Spawn a thread to print from out pipe
                        let stdout = process_handle.stdout.unwrap();
                        thread::scoped(move || {
                            let process_reader = StdOutIter{ out : stdout };

                            for output in process_reader {
                                print!("{}", output);
                            }
                        })
                    }
                };

                // Helper thread handles drop, joining on them.

            })
        }   // End of start_piped_process

    // This method to read from file and write to channel
    fn create_io_reader(channel : Sender<String>, file_name: Box<&str>)
        -> Result<JoinHandle>{
            // Create and validate file object. Return Error early if failure or spawn thread
            let path = Path::new(file_name.as_slice());
            let file = match File::open(&path) {
                Err(why) => return Err(why),
                Ok(f) => f,
            };
            // File created successfully, return thread that will be reading from it
            Ok( thread::spawn(move || {
                let f_iter = FileReadIter::new(file);
                for buffer_amt in f_iter {
                    channel.send(buffer_amt).unwrap();
                }
            }) )
        }

    // This method to read from channel and write to file
    fn create_io_writer(channel : Receiver<String>, file_name: Box<&str>)
        -> Result<JoinHandle>{
            // Create and validate file object. Return Error early if failure or spawn thread
            let path = Path::new(file_name.as_slice());
            let mut file = match File::create(&path) {
                Err(why) => return Err(why),
                Ok(f) => f,
            };
            // File created successfully, return thread that will be writing to it
            Ok( thread::spawn(move || {
                // Write data from channel until channel is closed (Err)
                loop {
                    match channel.recv() {
                        Ok(msg) => { file.write_all(msg.as_bytes()).unwrap(); }
                        Err(_) => { break; }    // Channel closed
                    }
                }
            }) )
        }

}   // End of impl GashCommand


/// A GashOperation is the basic unit of an operation, contains an operator ("echo") 
/// and a vector of operands (arguments to operator).
struct GashOperation {
    operator : Box<String>,
    operands: Box<Vec<String>>
}

impl GashOperation {

    fn new(operator : Box<& str>, operands : Box<Vec<& str>>) -> GashOperation {
        let operator = String::from_str(*operator);
        let mut operands_string = Vec::new();
        for op in operands.iter(){
            operands_string.push(String::from_str(op));
        }
        GashOperation{operator:Box::new(operator),operands:Box::new(operands_string)}
    }
    // Runs command with args
    // Returns handle to the Command after spawning it
    fn run_cmd(&self) -> Result<process::Child> { 
        process::Command::new((*self.operator).as_slice()).args(&*self.operands.as_slice())
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

/// FileReadIter for encapsulating reading an entire file BUFFER_SIZE bytes at a time
struct FileReadIter {
    file: File,
}

impl FileReadIter {
    fn new(file_handle : File) -> FileReadIter {
        FileReadIter { file : file_handle }
    }
}

impl<'a> Iterator for FileReadIter {
    type Item = String;

    /// each time next is called, BUFFER_SIZE bytes are read and returned as Some(String)
    /// None signals end of file (due to no data read or Err)
    fn next(& mut self) -> Option<String> {
        let mut buffer_array : [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
        let buffer = &mut buffer_array;

        let output_str = match self.file.read(buffer) {
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
