
use std::str::{Chars};





fn main() {
    let s = "test";
    let xs: [&str; 1] = [s];
    let b = "moo";
    //let g = gashCommand {op: s, args: b};
    println!("{}",'5'=='5');
    let mut m = parse_command_line("abc|def");
   // println!("{}",m.next().unwrap());


    //println!("{}",m.take(5));
        //let f = GashCommand::new(s,&xs);
}

struct GashCommand<'a> {
    op: & 'a str,
    args: Vec<& 'a str>
}

impl<'b> GashCommand<'b> {
    fn new(line : & 'b str) -> GashCommand<'b> {
        let mut lineIter = line.words();
        GashCommand {
            op: lineIter.next().unwrap(),
            args: lineIter.collect::<Vec<&str>>()
        }
    }
}

struct GashCommandParser<'a> {
    buffer: Chars<'a>
}

// Returns a fibonacci sequence generator
fn parse_command_line(line: &str) -> GashCommandParser {
    GashCommandParser {buffer:line.chars()}
}

// Implement 'Iterator' for 'Fibonacci'
impl<'b> Iterator for GashCommandParser<'b> {
    type Item = & 'b GashCommand<'b>;

    fn next(&mut self) -> Option<& 'b GashCommand> {
        let charIter = &mut self.buffer;
        let mut op = String::new();
        for c in charIter {
            op.push(c);
            if c=='|' {
                //let b = GashCommand::new(op.as_slice());
                
            }
        }
        return None;
    }
}

//string slice
//array of string slices
