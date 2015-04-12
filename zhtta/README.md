#Zhtta Server (Part 1)

- Authors: Michael Partridge, Eden Zik
- Course: CS146A - System Design
- Langauge: Rust
- Version: 1.0.0-nightly (2b01a37ec 2015-02-21) (built 2015-02-21)
- Date: 4/12/15

We present the first revision of Zhtta server, capable of responding to HTTP requests with dynamiclly loaded content as well as incorporating a safe visitor counter.

##Description

###Safe Counter
To implement a safe version of the visit counter, we use two rust constructs. The counter is wrapped in a mutex, which requires locking the mutex to retrieve the counter for incrementing and printing. This ensures that only one thread is modifying the counter at a time, preventing data races. In order to guarantee memory safety as this object is being accessed by various threads, we used an Arc pointer and passed copies of that pointer into each handler thread. The Arc pointer type atomically reference counts the accesses and ensures that the reference count is always accurate so that it will not be freed while a thread is using the counter. Arc requires that it be wrapped around an object implementing sync, which mutex does.

###Dynamic Content
Inspired by Apache dynamic content syntax, the Zhtta shell exeuction syntax is does not break HTML conventions and is hence embedded inside comments not parsed by a browser and hence not displayed in the DOM. 

A comment in HTML is as follows:

`<!-- -->`

Zhtta utilizes this syntax to execute a command, in the following form:

`<!--#exec cmd="$" -->`

Where `$` represents an arbitrary shell command to be executed by Gash.

The file extension for a dynamic file is `.shtml`, and when such a file is detected, it is passed on to `respond_with_dynamic_page`, which is a function reading the file.

At runtime, any HTML file read by Zhtta with the `.shtml` extension is parsed for comment syntax by `process_external_commands`, which looks for the `<!--` starting and `-->` ending structure. Any characters meeting this form that are mismatched will not be parsed further.

Once a comment is detected, it is passed on to `external_command`, which detects comments with the `#exec cmd=` syntax. If the syntax is found in a comment, the *entire* comment is replaced with the STDOUT output of the Gash command within the quotes, which is passed to `execute_gash`. Note that we used the provided implementation of Gash over our own due to the ease of passing external, single commands to it within the program.

For Zhtta to work, Gash needs to be at the root directory of the server (the same directory as the server binary). Otherwise, a debug log will instruct that Gash is not found.

##Building

To build Gash shell, navigate to the src directory and execute the following command:

`$rustc zhtta.rs`

This will create a shell binary in the current directory.

##Running

To run Zhtta, execute the following:

`$./zhtta`

The server will start at local port 4414.

##Logging

To run the web server with extensive error logging, run the following

`$RUST_LOG=debug ./zhtta`

This will present extensive log messages, for example when bad parsing occurs.