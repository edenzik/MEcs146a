//
// zhtta.rs
//
// Starting code for PA3
// Revised to run on Rust 1.0.0 nightly - built 02-21
//
// Note that this code has serious security risks!  You should not run it 
// on any system with access to sensitive files.
// 
// Brandeis University - cs146a Spring 2015
// Dimokritos Stamatakis and Brionne Godby
// Version 1.0

// To see debug! outputs set the RUST_LOG environment variable, e.g.: export RUST_LOG="zhtta=debug"

#![feature(rustc_private)]
#![feature(libc)]
#![feature(io)]
#![feature(old_io)]
#![feature(old_path)]
#![feature(os)]
#![feature(env)]
#![feature(core)]
#![feature(collections)]
#![feature(process)]
#![feature(std_misc)]
#![allow(non_camel_case_types)]
#![allow(unused_must_use)]
#[macro_use]
extern crate log;
extern crate libc;

use std::env;
// use std::old_path::posix::Path;
use std::borrow::ToOwned;
// use std::old_io::fs::PathExtensions;

extern crate getopts;
use getopts::{optopt, getopts};

mod web_server;
use web_server::WebServer;

mod http_request;
mod external_cmd;

// Server config
const IP : &'static str = "127.0.0.1";
const PORT : usize = 4414;
const WWW_DIR : &'static str = "./www";

fn get_args() -> (String, usize, String) {
    fn print_usage(program: &str) {
        println!("Usage: {} [options]", program);
        println!("--ip     \tIP address, \"{}\" by default.", IP);
        println!("--port   \tport number, \"{}\" by default.", PORT);
        println!("--www    \tworking directory, \"{}\" by default", WWW_DIR);
        println!("-h --help \tUsage");
    }

    // Begin processing program arguments and initiate the parameters.
    let mut args = env::args();
    let program = args.next().unwrap();

    let opts = [
        getopts::optopt("", "ip", "The IP address to bind to", "IP"),
        getopts::optopt("", "port", "The Port to bind to", "PORT"),
        getopts::optopt("", "www", "The www directory", "WWW_DIR"),
        getopts::optflag("h", "help", "Display help"),
        ];

    let matches = match getopts::getopts(&args.collect::<Vec<_>>(), &opts) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_err_msg()) }
    };

    if matches.opt_present("h") || matches.opt_present("help") {
        print_usage(program.as_slice());
        unsafe { libc::exit(1); }
    }

    let ip_str = if matches.opt_present("ip") {
        matches.opt_str("ip").expect("invalid ip address?").to_owned()
    } else {
        IP.to_owned()
    };

    let port:usize = if matches.opt_present("port") {
        let input_port = matches.opt_str("port").expect("Invalid port number?").trim().parse::<usize>().ok();
        match input_port {
            Some(port) => port,
            None => panic!("Invalid port number?"),
        }
    } else {
        PORT
    };

    let www_dir_str = if matches.opt_present("www") {
        matches.opt_str("www").expect("invalid www argument?") 
    } else { WWW_DIR.to_owned() };

    (ip_str, port, www_dir_str)    
}

fn main() {
    let (ip_str, port, www_dir_str) = get_args();
    let mut zhtta = WebServer::new(ip_str, port, www_dir_str);
    zhtta.run();
}