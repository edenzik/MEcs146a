use std::collections::hash_map::HashMap;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
fn main(){
    println!("ehl");
}


/// A GashOperation is the basic unit of an operation, contains an operator ("echo") 
/// and a vector of operands (arguments to operator).
struct ServerFileCache {
    path_string_to_cached_file: HashMap<String,CachedFile>,
    cached_file_ttl_heap: BinaryHeap<CachedFile>,
    capacity: u64,
    size: u64,
    latest: usize
}

/// The cached file struct keeps track of a file as a vector of bytes and its last modified date
struct CachedFile {
    ttl: usize,
    content: Vec<u8>,
    modified: u64
    
}

impl Ord for CachedFile {
    fn cmp(&self, other: &CachedFile) -> Ordering {
        // Notice that the we flip the ordering here
        other.ttl.cmp(&self.ttl)
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for CachedFile {
    fn partial_cmp(&self, other: &CachedFile) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for CachedFile {
    fn eq(&self, other: &CachedFile) -> bool {
        self.content == other.content
    }
}

impl Eq for CachedFile {}


impl ServerFileCache {
    /// Create new GashOperation by deep-copying string slices into internally referenced
    /// Strings so that this struct can be self-contained and passed into threads safely.
    fn new(capacity: u64) -> ServerFileCache {
        ServerFileCache{
            path_string_to_cached_file:HashMap::new(),
            capacity:capacity,
            cached_file_ttl_heap:BinaryHeap::new(),
            latest:0
        }
    }

    fn get(&self, path_string : String, modified : u64) -> Option<&CachedFile> {
        match self.path_string_to_cached_file.get(&path_string){
            Some(cached_file) if cached_file.modified == modified => {
                let mut current_ttl = cached_file.ttl;
                current_ttl += 1;
                Some(cached_file)
            }
            _ => None
        }
    }

    fn insert(&mut self, path_string : String, modified: u64, content: Vec<u8>){
        let cached_file = CachedFile{
            ttl: 0,
            content: content,
            modified: modified
        };
        self.path_string_to_cached_file.insert(path_string,cached_file);

        
    }

    fn free(&mut self, reduce_by : u64){
        
    }

}
