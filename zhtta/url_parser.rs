/// url_parser.rs
/// Mike Partridge and Eden Zik
/// CS146A - Principles of Computer Systems Design
/// April 2015

/// URL Parser module handles all parsing of request URLs including decisions related to
/// well-formed requests and variables passed for dynamic content.


use http_request::HTTPRequest;
use external_cmd::DynamicResponse;
use std::{env};
use std::old_io::fs::PathExtensions;


pub enum ZhttaURL {
    /// A counter page
    Counter,
    /// An error page
    Error(String),
    /// A static file request
    Static(HTTPRequest),
    /// A dynamic file request
    Dynamic(HTTPRequest, DynamicResponse),
    /// A badly parsed URL
    Bad
}

/// Implements the Zhtta URL class, which parses the incoming URL just as before. This
/// implementation allows for pattern matching based on the request type, and later on split the
/// url by the existence of the ? symbol for parameter passing, error pages, static pages, etc. We
/// then implement pattern matching to determine what page to serve.
impl ZhttaURL{
    pub fn new(request_str : String, peer_name: String) -> ZhttaURL {
        let mut request_str_split_iter = request_str.splitn(3, ' ');
        request_str_split_iter.next();
        match request_str_split_iter.next(){
            Some(url_path_str) => {
                let path_str = ".".to_string() + url_path_str;
                let mut path_obj = env::current_dir().unwrap();
                path_obj.push(path_str.clone().split('?').next().unwrap());
                let ext_str = match path_obj.extension_str() {
                    Some(e) => e,
                    None => "",
                };

                 if path_str.as_slice().eq("./"){
                    return ZhttaURL::Counter;
                }
                if !path_obj.exists() || path_obj.is_dir(){
                    return ZhttaURL::Error(String::from_str(path_obj.as_str().expect("invalid path")));
                }
                let request = HTTPRequest::new( peer_name, path_obj.clone() );
                if ext_str == "shtml" {
                    debug!("{}", url_path_str);
                    return ZhttaURL::Dynamic(request, DynamicResponse::new(url_path_str));
                }
                return ZhttaURL::Static(request);
            },
            None => ZhttaURL::Bad
        }
    }
}


