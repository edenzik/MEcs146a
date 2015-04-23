use std::collections::hash_map::HashMap;
use std::collections::VecDeque;

fn main(){
    println!("ehl");
}


/// A GashOperation is the basic unit of an operation, contains an operator ("echo") 
/// and a vector of operands (arguments to operator).
struct ServerFileCache {
    path_string_to_cached_file: HashMap<String,CachedFile>,
    ttl_queue: VecDeque<String>,
    capacity: usize,
    size: usize
}

/// The cached file struct keeps track of a file as a vector of bytes and its last modified date
struct CachedFile {
    content: Vec<u8>,
    modified: usize
}

impl CachedFile {
    fn new(content: Vec<u8>, modified: usize) -> CachedFile {
        CachedFile{
            content: content,
            modified: modified
        }
    }
    fn size(&self) -> usize{
        self.content.len()
    }
}


/// Implements a server file cache
impl ServerFileCache {
    fn new(capacity: usize) -> ServerFileCache {
        ServerFileCache{
            path_string_to_cached_file:HashMap::new(),
            ttl_queue: VecDeque::new(),
            capacity:capacity,
            size: 0
        }
    }

    fn size(&self) -> usize{
        self.size
    }

    fn capacity(&self) -> usize{
        self.capacity
    }

    fn free_space(&self) -> usize{
        self.size() - self.capacity()
    }

    fn get(&self, path_string : String, modified : usize) -> Option<&CachedFile> {
        match self.path_string_to_cached_file.get(&path_string){
            Some(cached_file) if cached_file.modified >= modified => Some(cached_file),
            _ => None
        }
    }

    fn insert(&mut self, path_string : String, modified: usize, content: Vec<u8>){
        let cached_file = CachedFile::new(content, modified);
        if cached_file.size() > self.capacity(){
            return;
        }
        let current_size = self.size();
        if cached_file.size() > self.size() {
            self.free(cached_file.size()-current_size);
        }
        self.enqueue(String::from_str(path_string.as_slice()));
        self.path_string_to_cached_file.insert(path_string,cached_file);
    }
    
    /// Insert to TTL Queue
    fn enqueue(&mut self, path_string: String){
        match self.find(path_string.as_slice()){
            Some(index) => {self.ttl_queue.remove(index);}
            None => {}
        }
        self.ttl_queue.push_front(path_string);
    }
    
    /// Finds the index of a specified element
    fn find(&mut self, path_string: &str) -> Option<usize>{
        self.ttl_queue.iter().position(|ele| *ele == path_string)  
    }
    
    /// Removes the least recently accessed element, returns its size
    fn remove_lru(&mut self) -> usize {
        match self.ttl_queue.pop_back(){
            Some(old_cache_path) => 
                match self.path_string_to_cached_file.remove(&old_cache_path){
                    Some(old_cache_file) => old_cache_file.size(),
                    None => 0
                },
            None => 0
        }
    }

    fn free(&mut self, space_to_free : usize){
        for _ in 0..self.ttl_queue.len(){
            self.size -= self.remove_lru();
            if self.free_space() >= space_to_free{
                return;
            }
        }
    }

}
