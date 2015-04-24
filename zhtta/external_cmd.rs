use std::process::{Command, Stdio};
use std::collections::hash_map::HashMap;


/// A dynamic response, which contains any arguments to replace in the commands 
pub struct DynamicResponse {
    replacement_map : HashMap<String,String>                    // A replacement map for arguments
}

/// Implements dynamic response
impl DynamicResponse {
    /// Initializes all arguments if those exist
    pub fn new(uri_string : &str) -> DynamicResponse {
        match DynamicResponse::parse_args(uri_string){
            Some(map) => DynamicResponse{replacement_map:map},
            None => DynamicResponse{replacement_map:HashMap::new()}
        }
    }

    /// Processes a dynamic response
    pub fn process(&self, source : &str) -> String{
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
                    output.push_str(source[temp_index .. start].as_slice());
                    output.push_str(self.external_command(source[start .. end].as_slice()).as_slice());
                    temp_index = end;
                },
                (_,end)   =>  {                                   
                    // A parsing error occurred, abort and return the original HTML for security
                    debug!("BAD PARSE: Dangling end comment string at position {}",end);
                    return String::from_str(source);
                }
            }        
        }
        output.push_str(&source[temp_index .. ]);               // Push the dangling end of the string
        output
    }

    fn parse_args(uri_string : &str) -> Option<HashMap<String,String>> {
        let mut uri_string_split_iter = uri_string.split('?');
        let mut args = HashMap::new();
        uri_string_split_iter.next();
        match  uri_string_split_iter.next(){
            Some(arg_pairs_str) => {
                for arg_pair in arg_pairs_str.split('&'){
                    let mut arg_pair_key_value_split_iter = arg_pair.split('=');
                    match arg_pair_key_value_split_iter.next(){
                        Some(key) => match arg_pair_key_value_split_iter.next(){
                            Some(value) => args.insert("$".to_string() + key,String::from_str(value)),
                            None => None
                        },
                        None => None
                    };
                }
                Some(args)

            },
            None => None
        }
    }

    /// Parses comment string with a command in it. Returns comment string verbatim if command not
    /// found, otherwise parses command and passes it to execute gash which carries it out.
    fn external_command(&self, comment : &str) -> String{          // Iterates through a comment
        match comment.match_indices("#exec cmd=\"").next(){     // Finds index of command execution, if exists
            Some((_,start)) => {
                match comment[start..].match_indices("\"").last(){
                    Some((end,_)) => self.execute_gash(self.replace(comment[start..start+end].as_slice()).as_slice()),//comment[start..start+end].as_slice()),       //Executes gash
                    None => {
                        debug!("BAD PARSE: No quote terminating command at position {}",start);
                        return String::from_str(comment);
                    }
                }
            },
            None => String::from_str(comment)        // Returns result
        }
    }

    fn replace(&self, raw_string : &str) -> String{
        let mut modified_string = String::new();
        for term in raw_string.words(){
            modified_string.push_str(self.replace_term(term).as_slice());
            modified_string.push(' ');
        }
        debug!("modified: {}", modified_string);
        modified_string
    }

    fn replace_term(&self, raw_string : &str) -> String{
        debug!("replacing {}", raw_string);
        match self.replacement_map.get(raw_string){
            Some(value) => String::from_str(value),
            None => String::from_str(raw_string)
        }
    }

    /// Runs external command and returns the output
    fn execute_gash(&self, command_string : &str) -> String {
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
}


