#![feature(process)]
#![feature(collections)]
#![feature(io)]


use std::process::{self, Command, Stdio};
use std::str;
use std::io::Read;

static IP : &'static str = "

<html><head><title>Hello, Rust!</title>
<style>body { background-color: #111; color: #FFEEAA }
    h1 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red}
    h2 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green}
</style></head>
<body><!---->
<h1>Greetings, Krusty!</h1><!---->
<h2>Date: <!--#exec cmd=\"date\" --></h2>
</body></html>


";


fn process_external_commands(source : &str) -> String {
    let mut start = source.match_indices("<!--");       //indexes of all comment start sequences
    let mut end = source.match_indices("-->");          //indexes of all comment end sequences

    let mut ranges = Vec::new();                        //index of all comment ranges

     loop {                                             //Iterate over starts and end sequences, add their beginning and end to ranges as pair (start,end)
        match start.next(){
            Some((x,_)) => match end.next(){
                Some((_,y)) => ranges.push((x,y)),
                None => break
            },
            None => break
        }
    }


    let mut output = String::new();                        //Resulting output

    
    let mut temp_index = 0;                             //Temporary index

    for range in ranges{                                //Iterate over ranges
        match range {
            (start,end) if start<end => {
                output.push_str(&source[temp_index .. start]);
                output.push_str(&external_command(&source[start .. end]));
                temp_index = end;
            },
            _   =>  {                                   //A parsing error occurred, abort and return the original HTML for security
                println!("parse error");
                return String::from_str(source);
            }
        }        
    }
    output.push_str(&source[temp_index .. ]);               //Push the dangling end of the string

    return output;
}

fn external_command(comment : &str) -> String{
    match comment.match_indices("#exec cmd=\"").next(){
        Some((_,start)) => {
            match comment[start..].match_indices("\"").next(){
                Some((end,_)) => return execute_gash(&comment[start..start+end]),
                None => return String::from_str(comment)
            }
        },
        None => return String::from_str(comment)
    }
}



fn main() {
    println!("{}",process_external_commands(IP));
}

fn execute_gash(command_string : &str) -> String {
    let args: &[_] = &["-c", &command_string];
        let cmd = Command::new("./gash").args(args).stdout(Stdio::capture()).spawn().unwrap();
        let iter = StdOutIter{ out : cmd.stdout.unwrap() };
        let mut result = String::new();
        for text in iter {
            result.push_str(&text.trim());
        }
        return result;

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
            Err(_) => { println!("Error: Output not UTF8 encoding. Read failed.");
            None }
        }

    }
}
