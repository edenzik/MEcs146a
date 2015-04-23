use std::collections::hash_map::HashMap;
use std::collections::VecDeque;

const LOAD_FACTOR : usize = 20;  /// Default initial allocated size of the HashMap used for the cache. Increase for more pre-allocation and less overhead.

/// Server File Cache structure keeps track of all the files in a map
pub struct ServerFileCache {
    path_string_to_cached_file: HashMap<String,CachedFile>,
    ttl_queue: VecDeque<String>,        // Time to live queue for each path
    capacity: usize,                    // Capacity of the cache
    size: usize                         // The current size of the cache
}

/// The cached file struct keeps track of a file as a vector of bytes and its last modified date
pub struct CachedFile {
    content: Vec<u8>,
    modified: u64
}

impl CachedFile {
    fn new(content: Vec<u8>, modified: u64) -> CachedFile {
        CachedFile{
            content: content,
            modified: modified
        }
    }
    pub fn size(&self) -> usize{
        self.content.len()
    }

    pub fn content(&self) -> &Vec<u8> {
        &self.content
    }
}


/// Implements a server file cache
impl ServerFileCache {
    pub fn new(capacity: usize) -> ServerFileCache {
        ServerFileCache{
            path_string_to_cached_file:HashMap::with_capacity(LOAD_FACTOR),
            ttl_queue: VecDeque::with_capacity(LOAD_FACTOR),
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
        self.capacity() - self.size()
    }

    pub fn get(&self, path_string : String, modified : u64) -> Option<&CachedFile> {
        match self.path_string_to_cached_file.get(&path_string){
            Some(cached_file) if cached_file.modified >= modified => {
                debug!("File {} was found in the cache", path_string);
                Some(cached_file)
            }
            Some(old_cached_file) => {
                debug!("File {} modified {} is old, version on disk modified {}",path_string, old_cached_file.modified, modified);
                None
            }
            _ => {
                debug!("File {} was not found in the cache",path_string);
                None
            }
        }
    }

    pub fn insert(&mut self, path_string : String, modified: u64, content: Vec<u8>){
        let cached_file = CachedFile::new(content, modified);
        if cached_file.size() > self.capacity(){
            debug!("File {} of size {} too big to fit in cache of size {}", path_string, cached_file.size(), self.capacity());
            return;
        }
        match self.path_string_to_cached_file.remove(&path_string){
            Some(old_cached_file) => self.size -= old_cached_file.size(),
            None => {}
        }
        let current_size = self.size();
        if cached_file.size() > self.free_space() {
            let size_diff = current_size - cached_file.size();
            debug!("There is not enough space for file {} of size {}, attempting to free {} of space from cache",path_string, cached_file.size(), size_diff);
            self.free(size_diff);
        }
        self.size += cached_file.size();
        debug!("Finished caching file {} of size {}. Size of cache: {}", path_string, cached_file.size(), self.size());
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
                    Some(old_cache_file) => {
                        debug!("Removing least recently used file in cache, {} of size {}", old_cache_path, old_cache_file.size());
                        old_cache_file.size()
                    }
                    None => 0
                },
            None => 0
        }
    }
    
    /// Frees space in the cache by the space_to_free parameter
    fn free(&mut self, space_to_free : usize){
        debug!("FREE Start: Freeing {} of space", space_to_free);
        for _ in 0..self.ttl_queue.len(){
            self.size -= self.remove_lru();
            if self.free_space() >= space_to_free{
                return;
            }
        }
        debug!("FREE End: Size is now {}", self.size());
    }

}
