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
        let stats = path.stat().unwrap();
        let path_string = String::from_str(path.as_str().unwrap());

        HTTPRequest { peer_name: peer_name, path: path, path_string: path_string, 
            size: stats.size, modified: stats.modified }
    }

    pub fn path_string(&self) -> String{
        self.path_string.clone()
    }

    pub fn size(&self) -> u64{
        self.size
    }

    pub fn path(&self) -> &Path{
        &self.path
    }

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

impl Eq for HTTPRequest {}

/// Makes HTTPRequests sortable by size of requested file
impl Ord for HTTPRequest {
    fn cmp(&self, other: &HTTPRequest) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
