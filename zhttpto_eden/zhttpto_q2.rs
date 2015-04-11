//
// PA1 - zhttpto.rs
// Eden Zik
// Question 2 Code
// 
// Running on Rust 1.0.0-nightly build 2015-02-21
// Code using a static global mutable variable with unsafe blocks to keep a running counter of the number of requests. Uses unsafe blocks to increment the variable. Prints the counter to the console.
//
// Brandeis University - cs146a - Spring 2015


use std::old_io::{Acceptor, Listener, TcpListener};
use std::str;
use std::thread::Thread;

static mut visitor_count: int = 0;              //Global mutable static variable

fn main() {
    let addr = "127.0.0.1:4414";

    let mut acceptor = TcpListener::bind(addr).unwrap().listen().unwrap();

    println!("Listening on [{}] ...", addr);

    for stream in acceptor.incoming() {
        match stream {
            Err(_) => (),
            Ok(mut stream) => {
                // Spawn a thread to handle the connection
                Thread::spawn(move|| {
                    match stream.peer_name() {
                        Err(_) => (),
                        Ok(pn) => {
                            unsafe {
                                println!("Received connection from: [{}] - Requests - {}", pn, visitor_count);
                                visitor_count += 1;
                            }
                        }
                    }

                    let mut buf = [0 ;500];
                    let _ = stream.read(&mut buf);
                    match str::from_utf8(&buf) {
                        Err(error) => println!("Received request error:\n{}", error),
                        Ok(body) => println!("Received request body:\n{}", body),
                    }

                    let response =
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
                         <doctype !html><html><head><title>Hello, Rust!</title>
                         <style>body { background-color: #111; color: #FFEEAA }
                                h1 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red}
                                h2 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green}
                         </style></head>
                         <body>
                         <h1>Greetings, Krusty!</h1>
                         </body></html>\r\n";
                    let _ = stream.write(response.as_bytes());
                    println!("Connection terminates.");
                });
            },
        }
    }

    drop(acceptor);
}