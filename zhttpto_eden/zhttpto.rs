//
// zhttpto.rs
//
// Starting code for PA1
// Running on Rust 1.0.0-nightly build 2015-02-21
//
// Note that this code has serious security risks! You should not run it
// on any system with access to sensitive files.
//
// Brandeis University - cs146a - Spring 2015


use std::old_io::{Acceptor, Listener, TcpListener};
use std::str;
use std::thread::Thread;
//static mut visitor_count: int = 0;

fn main() {
    let addr = "127.0.0.1:4414";
    let mut acceptor = TcpListener::bind(addr).unwrap().listen().unwrap();
    println!("Listening on [{}] ...", addr);
    let mut visitor_count = 0;
    for stream in acceptor.incoming() {
        match stream {
            Err(_) => (),
            Ok(mut stream) => {
                // Spawn a thread to handle the connection
                visitor_count += 1;
                Thread::spawn(move|| {
                    match stream.peer_name() {
                        Err(_) => (),
                        Ok(pn) => {
                            println!("Received connection from: [{}], Count - {}", pn, visitor_count);
                        }
                    }


                    let mut buf = [0 ;500];
                    let _ = stream.read(&mut buf);
                    match str::from_utf8(&buf) {
                        Err(error) => println!("Received request error:\n{}", error),
                        Ok(body) => println!("Recieved request body:\n{}", body),
                    }
                    //unsafe{
                        let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
                         <doctype !html><html><head><title>Hello, Rust!</title>
                         <style>body {{ background-color: #111; color: #FFEEAA }}
                                h1 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red}}
                                h2 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green}}
                         </style></head>
                         <body>
                         <h1>Greetings, Krusty! {}</h1>
                         </body></html>\r\n", visitor_count);

                        let _ = stream.write(response.as_bytes());
                   // }
                    println!("Connection terminates.");
                });
            },
        }
    }

    drop(acceptor);
}
