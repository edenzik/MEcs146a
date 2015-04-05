use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::thread::{JoinHandle, spawn};
use std::error::Error;


fn main() {
    // Create a path to the desired file
    let path = Path::new("hello.txt");
    let display = path.display();

    // Open the path in read-only mode, returns `IoResult<File>`
    let mut file = match File::open(&path) {
        // The `desc` field of `IoError` is a string that describes the error
        Err(why) => panic!("couldn't open {}: {}", display,
                           Error::description(&why)),
                           Ok(file) => file,
    };

    // Read the file contents into a string, returns `IoResult<String>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display,
                           Error::description(&why)),
                           Ok(_) => print!("{} contains:\n{}", display, s),
    }

    // `file` goes out of scope, and the "hello.txt" file gets closed
}

fn create_thread_io(channel : Sender<String>, file_name: Box<String>) -> Result<JoinHandle, &'static str>{
    let mut result = String::new();
    let t = spawn(move || {
        let path = Path::new(file_name.as_slice());
        let mut file = match File::open(&path) {
            Err(why) => Err("bad time reading file"),
            Ok(file) => Ok(file),
        };
        match file {
            Err(why) => "shit",
            Ok(f)   =>  f.read_to_string(&mut result),
        };
    });
    match channel.send(result) {
        Err(why) => Err("failed to send to channel"),
        Ok(_)   => Ok(t),
    };

}
