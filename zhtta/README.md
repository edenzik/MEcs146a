#Zhtta Server

- Authors: Michael Partridge, Eden Zik
- Course: CS146A - System Design
- Langauge: Rust
- Version: 1.0.0-nightly (2b01a37ec 2015-02-21) (built 2015-02-21)
- Date: 4/28/15

We present the second and final revision of Zhtta server, capable of responding to HTTP requests with dynamically loaded content as well as incorporating a safe visitor counter.

As we embarked upon the next series of improvements, we took into consideration the benchmarking performance of our old revision, and well as the Zhapto web server implemented earlier in the course.

In going about this new revision, we had several factors to consider - similar to those considered in a real, live system:
- Stability of the server when prompted by many requests
- Speed of the server, and minimal degradation over many requests
- Correctness, in the form that the request by the user is served in the most "expected" way.
- Safety of the server

All three of the considerations above are extremely important in a web server, that could be used to implement a real application. All four were considered to some extent, with the latter only being subtly implemented when possible.

Most of the trade-offs we faced in implementing Zhtta were performance and correctness, and a multitude of factors were taken into consideration when reasoning about highly parallel operations.

The new features implemented in this revision are as follows:
1. Smarter scheduling 
2. Responding to multiple response task
3. Shortest remaining processing time prioritization
4. Live file streaming, with a fixed buffer size passed onto the user in real time
5. Caching of files with a least-recently-used paging algorithm, to ensure cache size obeys fixed size constraints.
6. URL parameter passing into a server side Gash
7. Miscellaneous optimizations

## Structure

In order to allow for more readable code, more coherent following of the Rust guidelines, and 

## Description

### Safe Counter
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

## Smarter scheduling 


## Responding to multiple response task



## Shortest remaining processing time prioritization


##Building

To build Gash shell, navigate to the src directory and execute the following command:

`$make all`

This will create a shell binary in the current directory.

##Running

To run Zhtta, execute the following:

`$make run`

The server will start at local port 4414.

##Logging

To run the web server with extensive error logging, run the following

`$RUST_LOG=debug ./zhtta`

This will present extensive log messages, for example when bad parsing occurs.

