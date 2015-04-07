//
// PA1 - zhttpto.rs
// Eden Zik
// Question 4 Code
//
// Running on Rust 1.0.0-nightly build 2015-02-21
// Code using a local mutable variable without unsafe blocks to keep a running counter of the number of requests. Increments the counter outside of each thread for safe access. Prints the counter to the console. Returns a response to the user including the HTML string to display a greeting followed by the request number.
//
// Brandeis University - cs146a - Spring 2015


use std::old_io::{Acceptor, Listener, TcpListener, File, BufferedReader};
use std::str;
use std::thread::Thread;
use std::os;
//use std::io::{File, Open, ReadWrite};
//use std::io::File;

fn main() {
    let addr = "127.0.0.1:4414";

    let mut acceptor = TcpListener::bind(addr).unwrap().listen().unwrap();

    println!("Listening on [{}] ...", addr);

    let mut visitor_count = 0;                  //Local mutable variable

    for stream in acceptor.incoming() {
        match stream {
            Err(_) => (),
            Ok(mut stream) => {
                visitor_count += 1;             //Increments variable
                // Spawn a thread to handle the connection
                Thread::spawn(move|| {
                    match stream.peer_name() {
                        Err(_) => (),
                        Ok(pn) => {
                            println!("Received connection from: [{}] - Requests - {}", pn, visitor_count);
                        }
                    }


                    let mut buf = [0 ;500];
                    let _ = stream.read(&mut buf);
                    let mut content = "";
                    //let mut response = "";
                    //let mut data = from_utf8_owned(hw_file.read_to_end());
                    match str::from_utf8(&buf) {
                        Err(error) => println!("Received request error:\n{}", error),
                        Ok(body) => {
                            println!("Received request body:\n{}", body);
                            let mut split = body.split_str(" ");
                            split.next();
                            let rel_path = match split.next(){
                                Some(path) => Path::new(format!("{}",os::getcwd().unwrap().display()) + path),
                                None => panic!("Error while parsing request."),
                            };
                            let path = Path::new(rel_path);
                            match File::open(&path).read_to_string() {
                                Ok(string) => {
                                    stream.write(string.as_bytes());
                                }
                                Err(_) => {
                                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
                         <doctype !html><html><head><title>Hello, Rust!</title>
                         <style>body {{ background-color: #111; color: #FFEEAA }}
                                h1 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red}}
                                h2 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green}}
                         </style></head>
                         <body>
                         <h1>Greetings, Krusty! {}</h1>
                         </body></html>\r\n", visitor_count);
                                    stream.write(response.as_bytes());
                                }
                            };
                        }

                    }
                    println!("Connection terminates.");
                });
            },
        }
    }

    drop(acceptor);
}
