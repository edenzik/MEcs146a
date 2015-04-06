#![feature(collections)]
#![feature(io)]
#![feature(core)]
#![feature(process)]
#![feature(fs)]
#![feature(old_path)]
#![feature(str_words)]
#![feature(env)]


mod gash;

fn main(){
    let test_input = vec!["echo hello"];
    //println!("{:?}",run(test_input));
    assert_eq!(["hello"],run(test_input));

}

// Begins the REPL loop
fn run(mut input : Vec<&str>) -> Vec<String>{
    let mut history: Vec<String> = Vec::new();
    let mut output = Vec::new();
    // Main REPL loop, may spawn background jobs to finish
    loop {
        
        let trimmed_command_string = match input.pop(){
            Some(x) => x,
            None    => break
        };


        //recognizes escape character (arrow keys, etc) if non zero length
        if trimmed_command_string.len()>0 &&  trimmed_command_string.as_bytes()[0]==27 { 
            continue;    
        }

        let history_string = String::from_str(&trimmed_command_string);

        let gash_command_line = 
            gash::GashCommandLine::new( &trimmed_command_string, history.clone() );
        // Branch depending on parse of input
        match gash_command_line {
            // Special cases:
            gash::GashCommandLine::Empty => { continue; }  // Present another prompt
            gash::GashCommandLine::Exit => { break; }      // End REPL loop

            // Invalid input
            gash::GashCommandLine::UnsupportedCommand(msg) => { 
                output.push(format!("{}", msg)); 
                continue; 
            }

            // Invalid command
            gash::GashCommandLine::InvalidCommand(msg) => { 
                output.push(format!("gash: command not found: {}", msg)); 
                continue; 
            }


            // Else, run this well-formed batch of commands
            _ => { gash_command_line.run_batch(); }
        };

        // Add this history to the record
        history.push( history_string );
    }
    return output;
}


