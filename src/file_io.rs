use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::thread::{JoinHandle, spawn};
use std::error;
use std::str;


fn main() {
    // Create a path to the desired file
    let path = Path::new("hello.txt");
    let display = path.display();

    // Open the path in read-only mode, returns `IoResult<File>`
    let mut file = match File::open(&path) {
        // The `desc` field of `IoError` is a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", display,
                           error::Error::description(&why)),
                           Ok(file) => file,
    };

    // Read the file contents into a string, returns `IoResult<String>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display,
                           error::Error::description(&why)),
                           Ok(_) => print!("{} contains:\n{}", display, s),
    }

    // `file` goes out of scope, and the "hello.txt" file gets closed
}

// This method to read from file and write to channel
fn create_thread_io(channel : Sender<String>, file_name: Box<String>)
    -> Result<JoinHandle, io::Error>{
    
    let path = Path::new(file_name.as_slice());
    let file = match File::open(&path) {
        Err(why) => return Err(why),
        Ok(f) => f,
    };
    Ok( spawn(move || {
        let f_iter = FileReadIter{file: file};
        for buffer_amt in f_iter {
            channel.send(buffer_amt).unwrap();
        }
    }) )
}

// This method to read from channel and write to file
fn create_thread_io(channel : Receiver<String>, file_name: Box<String>)
    -> Result<JoinHandle, io::Error>{
    let path = Path::new(file_name.as_slice());
    let file = match File::open(&path) {
        Err(why) => return Err(why),
        Ok(f) => f,
    };
    Ok( spawn(move || {
        let f_iter = FileReadIter{file: file};
        for buffer_amt in f_iter {
            channel.send(buffer_amt).unwrap();
        }
    }) )
}

struct FileReadIter {
    file: File,
}

impl<'a> Iterator for FileReadIter {
    type Item = String;

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
