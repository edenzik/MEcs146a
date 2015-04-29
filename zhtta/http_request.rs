/// http_request.rs
/// Mike Partridge and Eden Zik
/// CS146A - Principles of Computer Systems Design
/// April 2015

/// The HTTP Request module encapsulates data about a requested file, primarily used for
/// static requests. Creation of a new request struct makes a system call to read inode data
/// for modified data and file size. Encapsulation of these requests ensures that only one
/// system call is made to check file statistics. These statistics are used for correct caching
/// behavior.
///
/// Requests implement ordering by size for shortest-request-first scheduling of non-cached
/// requests.

use std::cmp::Ordering;
use std::old_io::fs::PathExtensions;


pub struct HTTPRequest {
    pub peer_name: String,      // Use peer_name as the key to access TcpStream in hashmap. 
    pub path: Path,             // Path for the requested file
    pub path_string: String,    // String for cache lookup and pretty printing
    pub size: u64,              // File size for priority scheduling
    pub modified: u64           // Modified date for avoiding cache staleness
}

impl HTTPRequest {
    /// Constructor for HTTPRequest makes blocking call to system for file statistics
    pub fn new(peer_name: String, path: Path) -> HTTPRequest {
        let stats = match path.stat(){
            Ok(s) => s,
            Err(_) => panic!("Failed to get stats for file."),
        };

        let path_string = match path.as_str() {
            Some(s) => String::from_str(s),
            None => panic!("Failed to parse path to UTF-8 string."),
        };

        HTTPRequest { peer_name: peer_name, path: path, path_string: path_string, 
            size: stats.size, modified: stats.modified }
    }
    
    /// Returns the path string to the request
    pub fn path_string(&self) -> String{
        self.path_string.clone()
    }

    /// Size of the file served
    pub fn size(&self) -> u64{
        self.size
    }

    /// Path served
    pub fn path(&self) -> &Path{
        &self.path
    }

    /// Last modified date
    pub fn modified(&self) -> u64{
        self.modified
    }
}

/// Ordering for HTTPRequest states that larger files are 'smaller' i.e. lower in the queue
impl PartialOrd for HTTPRequest {
    fn partial_cmp(&self, other: &HTTPRequest) -> Option<Ordering> {
        match (self.size, other.size) {
            (x,y) if x > y => Some(Ordering::Less),
            (x,y) if x == y => Some(Ordering::Equal),
            _ => Some(Ordering::Greater)
        }
    }
}

impl PartialEq for HTTPRequest {
    fn eq(&self, other: &HTTPRequest) -> bool {
        self.size == other.size
    }
}

/// Needed by Rust
impl Eq for HTTPRequest {}

/// Makes HTTPRequests sortable by size of requested file
impl Ord for HTTPRequest {
    fn cmp(&self, other: &HTTPRequest) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
