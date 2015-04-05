use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::{JoinHandle, spawn};
use std::str;

// This method to read from file and write to channel
fn create_thread_io(channel : Sender<String>, file_name: Box<String>)
    -> Result<JoinHandle, io::Error>{
    // Create and validate file object. Return Error early if failure or spawn thread
    let path = Path::new(file_name.as_slice());
    let file = match File::open(&path) {
        Err(why) => return Err(why),
        Ok(f) => f,
    };
    // File created successfully, return thread that will be reading from it
    Ok( spawn(move || {
        let f_iter = FileReadIter::new(file);
        for buffer_amt in f_iter {
            channel.send(buffer_amt).unwrap();
        }
    }) )
}

// This method to read from channel and write to file
fn create_io_thread(channel : Receiver<String>, file_name: Box<String>)
    -> Result<JoinHandle, io::Error>{
    // Create and validate file object. Return Error early if failure or spawn thread
    let path = Path::new(file_name.as_slice());
    let mut file = match File::create(&path) {
        Err(why) => return Err(why),
        Ok(f) => f,
    };
    // File created successfully, return thread that will be writing to it
    Ok( spawn(move || {
        // Write data from channel until channel is closed (Err)
        loop {
            match channel.recv() {
                Ok(msg) => { file.write_all(msg.as_bytes()).unwrap(); }
                Err(_) => { break; }    // Channel closed
            }
        }
    }) )
}

/// FileReadIter for encapsulating reading an entire file 80 bytes at a time
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

    /// each time next is called, 80 bytes are read and returned as Some(String)
    /// None signals end of file (due to no data read or Err)
    fn next(& mut self) -> Option<String> {
        let mut buffer_array : [u8; 80] = [0; 80];
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
