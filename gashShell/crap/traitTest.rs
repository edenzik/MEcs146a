#![feature(env)]

use std::{env};

struct Argument {
    arg_text: String,
}

struct ArgIter {
    arg_list: String,
}

impl Iterator for ArgIter {
    type Item = Argument;

    fn next(&mut self) -> Option<Argument> {
        let next_arg = 
        let value = Argument { arg_text: }
    }
}

fn main() {
    let arg_list = env::args();
    let  mut how_many_args = 0;

    for arg in arg_list {
        println!("{}", arg);
        how_many_args += 1;
    }

    



}