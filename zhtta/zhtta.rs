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
#![feature(core)]
#![feature(collections)]
#![feature(process)]
#![feature(std_misc)]
#![allow(non_camel_case_types)]
#![allow(unused_must_use)]
#![allow(deprecated)]
#[macro_use]
extern crate log;
extern crate libc;

use std::io::Read;
use std::old_io::File;
use std::{os, str};
use std::old_path::posix::Path;
use std::collections::BinaryHeap;
use std::collections::hash_map::HashMap;
use std::cmp::Ordering;
use std::borrow::ToOwned;
use std::thread::Builder;
use std::old_io::fs::PathExtensions;
use std::old_io::{Acceptor, Listener};

extern crate getopts;
use getopts::{optopt, getopts};

use std::sync::{Arc, Mutex, Semaphore};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;

use std::process::{Command, Stdio};




const SERVER_NAME : &'static str = "Zhtta Version 1.0";

// Server config
const IP : &'static str = "127.0.0.1";
const PORT : usize = 4414;
const WWW_DIR : &'static str = "./www";

// Tunable parameters
const REQ_HANDLER_COUNT : isize = 20;   // Max number of file request handler threads
const BUFFER_SIZE : usize = 512;        // Size of file buffer to send (bytes)
const CACHE_CAPACITY: u64 = 500000000;  // Size of file cache (bytes)

// Static responses
const HTTP_OK : &'static str = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n";
const HTTP_BAD : &'static str = "HTTP/1.1 404 Not Found\r\n\r\n";

const COUNTER_STYLE : &'static str = "<doctype !html><html><head><title>Hello, Rust!</title>
             <style>body { background-color: #884414; color: #FFEEAA}
                    h1 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red }
                    h2 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green }
             </style></head>
             <body>";


struct HTTP_Request {
    peer_name: String,      // Use peer_name as the key to access TcpStream in hashmap. 
    path: Path,             // Path for the requested file
    path_string: String,    // String for cache lookup and pretty printing
    size: u64,              // File size for priority scheduling
    modified: u64           // Modified date for avoiding cache staleness
}

impl HTTP_Request {
    /// Constructor for HTTP_Request makes blocking call to system for file statistics
    fn new(peer_name: String, path: Path) -> HTTP_Request {
        let stats = path.stat().unwrap();
        let path_string = String::from_str(path.as_str().unwrap());

        HTTP_Request { peer_name: peer_name, path: path, path_string: path_string, 
            size: stats.size, modified: stats.modified }
    }
}

/// Ordering for HTTP_Request states that larger files are 'smaller' i.e. lower in the queue
impl PartialOrd for HTTP_Request {
    fn partial_cmp(&self, other: &HTTP_Request) -> Option<Ordering> {
        match (self.size, other.size) {
            (x,y) if x > y => Some(Ordering::Less),
            (x,y) if x == y => Some(Ordering::Equal),
            _ => Some(Ordering::Greater)
        }
    }
}

impl PartialEq for HTTP_Request {
    fn eq(&self, other: &HTTP_Request) -> bool {
        self.size == other.size
    }
}

impl Eq for HTTP_Request {}

/// Makes HTTP_Requests sortable by size of requested file
impl Ord for HTTP_Request {
    fn cmp(&self, other: &HTTP_Request) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// The cached file struct keeps track of a file as a vector of bytes and its last modified date
struct CachedFile {
    modified: u64,
    file: Vec<u8>
}

/// Web server struct stores properties of the web server, such as the counter, the IP, the port,
/// directory path, and others.
/// The file caches is kept as a HashMap of (file path) -> (CachedFile struct)
struct WebServer {
    ip: String,
    port: usize,
    www_dir_path: Path,
    visitor_count: usize,

    request_queue_arc: Arc<Mutex<BinaryHeap<HTTP_Request>>>,
    stream_map_arc: Arc<Mutex<HashMap<String, std::old_io::net::tcp::TcpStream>>>,
    file_cache: Arc<Mutex<HashMap<String,CachedFile>>>,             // A HashMap of file caches 
    cache_size: Arc<Mutex<u64>>,                                    // Keeps track of cache size

    notify_rx: Receiver<()>,
    notify_tx: Sender<()>
}

impl WebServer {
    fn new(ip: String, port: usize, www_dir: String) -> WebServer {
        let (notify_tx, notify_rx) = channel();
        let www_dir_path = Path::new(www_dir);
        os::change_dir(&www_dir_path);

        WebServer {
            ip:ip,
            port: port,
            www_dir_path: www_dir_path,
            visitor_count: 0,

            request_queue_arc: Arc::new(Mutex::new(BinaryHeap::new())),
            stream_map_arc: Arc::new(Mutex::new(HashMap::new())),
            file_cache: Arc::new(Mutex::new(HashMap::new())),               // Initializes file cache
            cache_size: Arc::new(Mutex::new(0)),                            // Initializes cache size

            notify_rx: notify_rx,
            notify_tx: notify_tx,
        }
    }

    fn run(&mut self) {
        self.listen();
        self.dequeue_static_file_request();
    }

    fn listen(&mut self) {
        let addr = String::from_str(format!("{}:{}", self.ip, self.port).as_slice());
        let www_dir_path_str = self.www_dir_path.clone();
        let request_queue_arc = self.request_queue_arc.clone();
        let notify_tx = self.notify_tx.clone();
        let file_cache_arc = self.file_cache.clone();
        let stream_map_arc = self.stream_map_arc.clone();
        let visitor_count = self.visitor_count;         // Clone a local copy of visitor_count


        Builder::new().name("Listener".to_string()).spawn(move|| {
            let listener = std::old_io::TcpListener::bind(addr.as_slice()).unwrap();

            // Make a mutex wrapped by a reference counter of visitor_count
            let visitor_count = Arc::new(Mutex::new(visitor_count)); 
            let mut acceptor = listener.listen().unwrap();
            println!("{} listening on {} (serving from: {}).", 
                     SERVER_NAME, addr, www_dir_path_str.as_str().unwrap());
            for stream_raw in acceptor.incoming() {
                // Make a local copy of the Arc (increases its internal count)
                let visitor_count = visitor_count.clone();

                let (queue_tx, queue_rx) = channel();
                queue_tx.send(request_queue_arc.clone());

                let notify_chan = notify_tx.clone();
                let stream_map_arc = stream_map_arc.clone();

                let file_cache_arc = file_cache_arc.clone();

                // Spawn a task to handle the connection.
                Builder::new().name("Handler".to_string()).spawn(move|| {
                    let (visit_tx, visit_rx) = channel();
                    {   // Lock visitor_count inside scope to unlock when done
                        // Acquire lock on visitor_count, block until lock can be held
                        let mut visitor_count = match visitor_count.lock() {
                            Ok(count) => count,
                            Err(_) => panic!("Error getting lock for visit count."),
                        };
                        *visitor_count += 1;      // Increment visitor_count
                        visit_tx.send(visitor_count.clone()).unwrap();
                    }
                    let this_count = visit_rx.recv().unwrap();

                    let request_queue_arc = queue_rx.recv().unwrap();

                    let mut stream = match stream_raw {
                        Ok(s) => {s}
                        Err(e) => { panic!("Error getting the listener stream! {}", e) }
                    };
                    let peer_name = WebServer::get_peer_name(&mut stream);
                    debug!("Got connection from {}", peer_name);

                    let mut buf: [u8;500] = [0;500];
                    stream.read(&mut buf);
                    let request_str = match str::from_utf8(&buf){
                        Ok(s) => s,
                        Err(e)=> panic!("Error reading from the listener stream! {}", e),
                    };
                    debug!("Request:\n{}", request_str);

                    let req_group: Vec<&str> = request_str.splitn(3, ' ').collect();
                    if req_group.len() > 2 {
                        let path_str = ".".to_string() + req_group[1];
                        let mut path_obj = os::getcwd().unwrap();
                        path_obj.push(path_str.clone());
                        let ext_str = match path_obj.extension_str() {
                            Some(e) => e,
                            None => "",
                        };

                        debug!("Requested path: [{}]", path_obj.as_str().expect("error"));
                        debug!("Requested path: [{}]", path_str);

                        if path_str.as_slice().eq("./")  {
                            debug!("===== Counter Page request =====");
                            WebServer::respond_with_counter_page(stream, this_count);
                            debug!("=====Terminated connection from [{}].=====", peer_name);
                        }  else if !path_obj.exists() || path_obj.is_dir() {
                            debug!("===== Error page request =====");
                            WebServer::respond_with_error_page(stream, &path_obj);
                            debug!("=====Terminated connection from [{}].=====", peer_name);
                        } else if ext_str == "shtml" { // Dynamic web pages.
                            debug!("===== Dynamic Page request =====");
                            WebServer::respond_with_dynamic_page(stream, &path_obj);
                            debug!("=====Terminated connection from [{}].=====", peer_name);
                        } else { 
                            debug!("===== Static Page request =====");
                            // Create an HTTP_Request object for either cache or non-cache serving
                            let req = HTTP_Request::new( peer_name, path_obj.clone() );

                            let file_cache = file_cache_arc.lock().unwrap();

                            match file_cache.get(&req.path_string){
                                Some(cached_file) => WebServer::respond_with_static_cached_file(stream,cached_file),  
                                None => WebServer::enqueue_static_file_request(stream, req, stream_map_arc, request_queue_arc, notify_chan),
                            }     
                        }
                    }
                });
            }
        });
    }

    fn respond_with_static_cached_file(mut stream: std::old_io::net::tcp::TcpStream, cached_file: &CachedFile) {
        stream.write(HTTP_OK.as_bytes());
        debug!("Responding with file from cache");
        stream.write(&cached_file.file);
    }


    fn respond_with_error_page(stream: std::old_io::net::tcp::TcpStream, path: &Path) {
        let mut stream = stream;
        let msg: String= format!("Cannot open: {}", path.as_str().expect("invalid path"));
        stream.write(HTTP_BAD.as_bytes());
        stream.write(msg.as_bytes());
    }

    fn respond_with_counter_page(stream: std::old_io::net::tcp::TcpStream, visitor_count: usize) {
        let mut stream = stream;
        let response: String = 
            format!("{}{}<h1>Greetings, Krusty!</h1><h2>Visitor count: {}</h2></body></html>\r\n", 
                    HTTP_OK, COUNTER_STYLE, 
                    visitor_count);     //print visitor count
        debug!("Responding to counter request");
        stream.write(response.as_bytes());
    }

    /// Initializes a buffer, writes BUFFER_SIZE segments of file to that buffer
    /// Implements file caching using a HashMap, with the file path as the key.
    /// Serves static file as live streams, reading off a chunk of a file and sending it to a
    /// client.
    /// Adds all files read but not in cache to the cache, with the exception of files too big for
    /// the cache.
    fn respond_with_static_file(cache_arc: Arc<Mutex<HashMap<String,CachedFile>>>,
        cache_size_arc : Arc<Mutex<u64>>, mut stream: std::old_io::net::tcp::TcpStream, 
        request: HTTP_Request, sem: Arc<Semaphore>) {

        let file_reader = File::open(&request.path).unwrap();

        debug!("Serving static file {}", request.path_string);

        Builder::new().name("Responder".to_string()).spawn(move|| {  // Builds threads
            let mut cache = cache_arc.lock().unwrap();               // Locks the cache
            let mut cache_size = cache_size_arc.lock().unwrap();     // Locks the size of the cache
            let mut file_data = Vec::new();                          // Initializes a new vector of the file to be read
            debug!("Checking cache of size {} for file {}", *cache_size, request.path_string);

            stream.write(HTTP_OK.as_bytes());

            let mut file_reader = file_reader;
            let mut buf : [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
            loop {
                match file_reader.read(&mut buf) {
                    Ok(length) if length==0 => break,
                    Ok(_)   => {},                      // Continue if buffer not empty
                    Err(_)  => break
                };
                file_data.push_all(&buf);
                *cache_size += buf.len() as u64;
                stream.write(&mut buf);
            }
            
            debug!("Cached file {} of size {}, cache size {}", request.path_string, request.size, *cache_size);
            cache.insert(request.path_string, 
                CachedFile{modified: request.modified, file: file_data});

            sem.release();          // Releases semaphore to allow another Responder thread to spawn

            // Closes stream automatically.
            debug!("=====Terminated connection from [{}].=====", request.peer_name);
        });
    }

    // Server-side gashing.
    fn respond_with_dynamic_page(stream: std::old_io::net::tcp::TcpStream, path: &Path) {
        let mut stream = stream;
        // Open file for dynamic content
        let mut file_reader = match File::open(path) {
            Ok(file) => file,
            Err(_) => panic!("Error opening dynamic page file."),
        };
        stream.write(HTTP_OK.as_bytes());
        // Read file as bytes
        let file_content_bytes = match file_reader.read_to_end() {
            Ok(bytes) => bytes,
            Err(_) => panic!("Error reading dynamic page file as bytes."),
        };
        // Convert file bytes to string
        let file_content = match String::from_utf8(file_content_bytes) {
            Ok(string) => string,
            Err(_) => panic!("Error converting file content to string."),
        };
        // Process dynamic content and write to output stream
        let processed_output = process_external_commands(file_content.as_slice());
        stream.write(processed_output.as_bytes());
    }

    fn enqueue_static_file_request(stream: std::old_io::net::tcp::TcpStream, req: HTTP_Request, 
       stream_map_arc: Arc<Mutex<HashMap<String, std::old_io::net::tcp::TcpStream>>>, 
       req_queue_arc: Arc<Mutex<BinaryHeap<HTTP_Request>>>, notify_chan: Sender<()>) {
        // Save stream in hashmap for later response.
        let (stream_tx, stream_rx) = channel();
        stream_tx.send(stream);
        let stream = match stream_rx.recv(){
           Ok(s) => s,
           Err(e) => panic!("There was an error while receiving from the stream channel! {}", e),
        };

        let local_stream_map = stream_map_arc.clone();
        {   // make sure we request the lock inside a block with different scope,
            // so that we give it back at the end of that block
            let mut local_stream_map = local_stream_map.lock().unwrap();
            local_stream_map.insert(req.peer_name.clone(), stream);
        }

        // Enqueue the HTTP request
        let (req_tx, req_rx) = channel();
        req_tx.send(req);

        debug!("Waiting for queue mutex lock.");

        let local_req_queue = req_queue_arc.clone();
        {   // make sure we request the lock inside a block with different scope, 
            // so that we give it back at the end of that block
            let mut local_req_queue = local_req_queue.lock().unwrap();
            let req: HTTP_Request = match req_rx.recv(){
                Ok(s) => s,
                Err(e) => panic!("There was an error while receiving from the request channel! {}", e),
            };
            local_req_queue.push(req);
            debug!("A new request enqueued, now the length of queue is {}.", local_req_queue.len());

            notify_chan.send(()); // Send incoming notification to responder task. 
        }
    }

    /// Dequeues file request prioritized by smallest file requested
    fn dequeue_static_file_request(&mut self) {
        let req_semaphore_arc = Arc::new(Semaphore::new(REQ_HANDLER_COUNT));

        let req_queue_get = self.request_queue_arc.clone();
        let stream_map_get = self.stream_map_arc.clone();

        // Receiver<> cannot be sent to another task. So we have to make this task as the main task
        // that can access self.notify_rx.
        let (request_tx, request_rx) = channel();
        loop {
            self.notify_rx.recv();    // waiting for new request enqueued.
            {   // make sure we request the lock inside a block with different scope, so that we
                // give it back at the end of that block
                let mut req_queue = req_queue_get.lock().unwrap();
                if req_queue.len() > 0 {
                    let req = req_queue.pop().unwrap();  // Removes request associated with smallest file in queue
                    debug!("A new request dequeued, now the length of queue is {}.", req_queue.len());
                    request_tx.send(req);
                }
            }

            // Get request from internal channel
            let request = match request_rx.recv(){
                Ok(s) => s,
                Err(e) => panic!("There was an error while receiving from the request channel! {}", e),
            };

            // Get stream from hashmap.
            let (stream_tx, stream_rx) = channel();
            {   // make sure we request the lock inside a block with different scope,
                // so that we give it back at the end of that block
                let mut stream_map = stream_map_get.lock().unwrap();
                let stream = stream_map.remove(&request.peer_name).expect("No option tcpstream found in stream map.");
                stream_tx.send(stream);
            }
            let stream = match stream_rx.recv(){
                Ok(s) => s,
                Err(e) => panic!("There was an error while receiving from the stream channel! {}", e),
            };

            // Semaphore ensures that we do not serve too many concurrent requests
            req_semaphore_arc.acquire();

            WebServer::respond_with_static_file(self.file_cache.clone(), self.cache_size.clone(), 
                stream, request, req_semaphore_arc.clone());
        }
    }

    fn get_peer_name(stream: &mut std::old_io::net::tcp::TcpStream) -> String{
        match stream.peer_name(){
            Ok(s) => {format!("{}:{}", s.ip, s.port)}
            Err(e) => {panic!("Error while getting the stream name! {}", e)}
        }
    }
}

fn get_args() -> (String, usize, String) {
    fn print_usage(program: &str) {
        println!("Usage: {} [options]", program);
        println!("--ip     \tIP address, \"{}\" by default.", IP);
        println!("--port   \tport number, \"{}\" by default.", PORT);
        println!("--www    \tworking directory, \"{}\" by default", WWW_DIR);
        println!("-h --help \tUsage");
    }

    // Begin processing program arguments and initiate the parameters.
    let args = os::args();
    let program = args[0].clone();

    let opts = [
        getopts::optopt("", "ip", "The IP address to bind to", "IP"),
        getopts::optopt("", "port", "The Port to bind to", "PORT"),
        getopts::optopt("", "www", "The www directory", "WWW_DIR"),
        getopts::optflag("h", "help", "Display help"),
        ];

    let matches = match getopts::getopts(args.tail(), &opts) {
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

/// Fill page with dynamically requested content by parsing comment syntax.
fn process_external_commands(source : &str) -> String {
    let mut start = source.match_indices("<!--");       // indexes of all comment start sequences
    let mut end = source.match_indices("-->");          // indexes of all comment end sequences

    let mut ranges = Vec::new();                        // index of all comment ranges

    loop {                                             
        // Iterate over starts and end sequences, add their beginning and end to ranges as pair (start,end)
        match start.next(){
            Some((head,_)) => match end.next(){
                Some((_,tail)) => ranges.push((head,tail)),
                None => {
                    debug!("BAD PARSE: Missing end of comment string in position {}", head);
                    break;
                }
            },
            None => break
        }
    }

    let mut output = String::new();                     // Resulting output

    let mut temp_index = 0;                             // Temporary index

    for range in ranges{                                // Iterate over ranges
        match range {
            (start,end) if start<end => {
                output.push_str(&source[temp_index .. start]);
                output.push_str(&external_command(&source[start .. end]));
                temp_index = end;
            },
            (_,end)   =>  {                                   
                //vA parsing error occurred, abort and return the original HTML for security
                debug!("BAD PARSE: Dangling end comment string at position {}",end);
                return String::from_str(source);
            }
        }        
    }
    output.push_str(&source[temp_index .. ]);               // Push the dangling end of the string
    output
}

/// Parses comment string with a command in it. Returns comment string verbatim if command not
/// found, otherwise parses command and passes it to execute gash which carries it out.
fn external_command(comment : &str) -> String{          // Iterates through a comment
    match comment.match_indices("#exec cmd=\"").next(){     // Finds index of command execution, if exists
        Some((_,start)) => {
            match comment[start..].match_indices("\"").last(){
                Some((end,_)) => execute_gash(&comment[start..start+end]),       //Executes gash
                None => {
                    debug!("BAD PARSE: No quote terminating command at position {}",start);
                    return String::from_str(comment);
                }
            }
        },
        None => String::from_str(comment)        // Returns result
    }
}

/// Runs external command and returns the output
fn execute_gash(command_string : &str) -> String {
    let args: &[_] = &["-c", &command_string];
    let cmd = match Command::new("../gash").args(args).stdout(Stdio::capture()).output() {
        Ok(c) => c,
        Err(_) => {
            debug!("ERROR: failed to spawn gash command to handle dynamic content, is gash binary present at top level directory?");
            return String::from_str(command_string);
        }
    };
    String::from_utf8(cmd.stdout).unwrap()
}
