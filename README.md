=== Gash shell ===
Authors: Michael Partridge, Eden Zik
Course: CS146A - System Design
Langauge: Rust
Version: 1.0.0-nightly (2b01a37ec 2015-02-21) (built 2015-02-21)
Date: 4/7/15

We present Gash shell, a concurrent, multi threaded Unix shell implementation in the Rust programming language with emphasis on safety and simplicity.

== Description ==

Rust is a programming language emphasizing safe design and fast performance. Its support for concurrent, safe inter thread communication makes it ideal for creating a modern shell and a great exercise in systems programming.

Using Rust primitives we managed to achieve a safe and efficient shell implementing the following features:

- External system calls to underlying shell
	- Background processes
	- Foreground processes
- History to view previous commands in the shell
- cd to change directory
- Exit to quit
- File redirection
	- IN
	- OUT
- Piping between processes

The following features present in the unix shell are not supported:

- Multiple redirections with <>
- Redirection between stdin, stdout, and stderr such as 2>&1
- Batching with || and &&

The shell is designed to recover from any predictable error with a helpful message and return the user a new prompt. 

These errors include files and directories not found, inability to create files, commands not found in the underlying system, etc.

We utilized the Rust type system to enable safe development model ensuring predictable behavior under almost any case. All functionality was split into the following files:

- shell.rs
- gash.rs

The Shell struct in the shell.rs file is an effective encapsulation of basic user interactions, including reading from STDIN and displaying a prompt. The Shell runs a Read-Eval-Print loop (REPL) which dispatches a user command and creates a new GashCommandLine struct using the user command string to construct it.

A shell works on line by line basis - and we structured ours accordingly. 

The GashCommandLine struct encapsulates the behavior of parsing user line input and splitting it on pipes. 

We utilized the powerful type system in Rust to make GashCommandLine an Algebraic Data Type (ADT) which we can then use for the purpose of propagating command lines containing a bad command, background commands (ones ending in &), command lines starting with an exit, or command lines that are not supported by our system (&& or ||).

This pattern was followed in GashCommand, which is also enumerated to enable pattern matching.

Using channels, we were able to route data amongst threads. Each transfer was utilized using three threads:
- External process thread containing an actual system Command (form the underlying OS)
- In thread passing information from a previous filter to the command
- Out thread passing information from the command to the next filter

<<<<Write more  about threading here>>>

== Building ==
To build Gash shell, navigate to the src directory and execute the following command:

`$rustc shell.rs`

This will create a shell binary in the current directory.

== Running ==
To run Gash shell, execute the following:

`$./shell`

A shell will promptly appear.

== Testing ==
To run basic unit tests, navigate to the src directory and execute the following command:

`$rustc --test shell.rs`

Run all tests by executing the resulting binary.

