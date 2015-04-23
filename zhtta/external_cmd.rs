use std::process::{Command, Stdio};

/// Fill page with dynamically requested content by parsing comment syntax.
pub fn process(source : &str) -> String {
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
