use std::sync::{Arc, Mutex, Semaphore};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc::channel;

use std::thread::Builder;
use std::{env, os, str};

use std::old_io::File;
use std::old_io::{ Acceptor, Listener, TcpListener };
use std::old_io::fs::PathExtensions;
use std::old_io::net::tcp::TcpStream;

use std::old_path::posix::Path;

use std::io::Read;

use std::collections::BinaryHeap;
use std::collections::hash_map::HashMap;

use http_request::HTTP_Request;
use external_cmd::process;

const SERVER_NAME : &'static str = "Zhtta Version 1.0";

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

/// The cached file struct keeps track of a file as a vector of bytes and its last modified date
struct CachedFile {
    modified: u64,
    file: Vec<u8>
}

/// Web server struct stores properties of the web server, such as the counter, the IP, the port,
/// directory path, and others.
/// The file caches is kept as a HashMap of (file path) -> (CachedFile struct)
pub struct WebServer {
    ip: String,
    port: usize,
    www_dir_path: Path,
    visitor_count: usize,

    request_queue_arc: Arc<Mutex<BinaryHeap<HTTP_Request>>>,
    stream_map_arc: Arc<Mutex<HashMap<String, TcpStream>>>,
    file_cache: Arc<Mutex<HashMap<String,CachedFile>>>,             // A HashMap of file caches 
    cache_size: Arc<Mutex<u64>>,                                    // Keeps track of cache size

    notify_rx: Receiver<()>,
    notify_tx: Sender<()>
}

impl WebServer {
    pub fn new(ip: String, port: usize, www_dir: String) -> WebServer {
        let (notify_tx, notify_rx) = channel();
        let www_dir_path = Path::new(www_dir);
        env::set_current_dir(&www_dir_path);

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

    pub fn run(&mut self) {
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
            let listener = TcpListener::bind(addr.as_slice()).unwrap();

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

    fn respond_with_static_cached_file(mut stream: TcpStream, cached_file: &CachedFile) {
        stream.write(HTTP_OK.as_bytes());
        debug!("Responding with file from cache");
        stream.write(&cached_file.file);
    }


    fn respond_with_error_page(stream: TcpStream, path: &Path) {
        let mut stream = stream;
        let msg: String= format!("Cannot open: {}", path.as_str().expect("invalid path"));
        stream.write(HTTP_BAD.as_bytes());
        stream.write(msg.as_bytes());
    }

    fn respond_with_counter_page(stream: TcpStream, visitor_count: usize) {
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
        cache_size_arc : Arc<Mutex<u64>>, mut stream: TcpStream, 
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
    fn respond_with_dynamic_page(stream: TcpStream, path: &Path) {
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
        let processed_output = process(file_content.as_slice());
        stream.write(processed_output.as_bytes());
    }

    fn enqueue_static_file_request(stream: TcpStream, req: HTTP_Request, 
       stream_map_arc: Arc<Mutex<HashMap<String, TcpStream>>>, 
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

    fn get_peer_name(stream: &mut TcpStream) -> String{
        match stream.peer_name(){
            Ok(s) => {format!("{}:{}", s.ip, s.port)}
            Err(e) => {panic!("Error while getting the stream name! {}", e)}
        }
    }
}