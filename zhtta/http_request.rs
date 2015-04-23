use std::cmp::Ordering;
use std::old_io::fs::PathExtensions;


pub struct HTTP_Request {
    pub peer_name: String,      // Use peer_name as the key to access TcpStream in hashmap. 
    pub path: Path,             // Path for the requested file
    pub path_string: String,    // String for cache lookup and pretty printing
    pub size: u64,              // File size for priority scheduling
    pub modified: u64           // Modified date for avoiding cache staleness
}

impl HTTP_Request {
    /// Constructor for HTTP_Request makes blocking call to system for file statistics
    pub fn new(peer_name: String, path: Path) -> HTTP_Request {
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