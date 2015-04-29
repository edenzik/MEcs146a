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

All four of the considerations above are extremely important in a web server, that could be used to implement a real application. All four were considered to some extent, with the latter only being subtly implemented when possible.

Most of the trade-offs we faced in implementing Zhtta were performance and correctness, and a multitude of factors were taken into consideration when reasoning about highly parallel operations.

The new features implemented in this revision are as follows:
1. Responding to multiple response task
2. Shortest remaining processing time prioritization
3. Live file streaming, with a fixed buffer size passed onto the user in real time
4. Caching of files with a least-recently-used paging algorithm, to ensure cache size obeys fixed size constraints.
5. URL parameter passing into a server side Gash
6. Miscellaneous optimizations

## Structure


In order to allow for more readable code, more coherent following of the Rust guidelines, and better concurrent work, our server has been split into several modules. Descriptions of the functionality of individual modules are included as block comments at the beginning of each file, so only a list of modules is included here. Modules are zhtta, web_server, http_request, external_cmd, server_file_cache, and url_parser. Modules expose only required public fields and functions to maximize encapsulation.

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

### Responding to multiple response task

To handle serving multiple files simultaneously, we spawn a thread to serve each file after dequeuing the request from the request queue. This thread will then block on file I/O independently of serving other requests. To limit the number of requests served simultaneously (threads incur some overhead and interfere with each other) the request serving thread first grabs a semaphore before spawning a child thread. This limits the number of child threads by the size of the semaphore and the request serving thread will block until a child thread dies and it can spawn another. Child threads release the semaphore after serving the request, just before they finish. This change significantly improved our test run time both in total and in average response time. Total time was reduced by over half.

### Reducing Median Latency 

We changed the request queue into an priority queue, ordered by the size of the files (as reported by the OS). The current (as of our version) priority queue in Rust is a binary heap. We implemented an ordering on the HTTP_Request object to support this. Thus, whenever a dequeue is performed on the request queue, this will return the shortest (expected) file to be served and prioritize that file. Note that due to the way caching is implemented, files served directly from the cache are never queued at all and thus our scheduling does not have to consider those files. This change improved overall performance of the test run, but only slightly. It had a higher impact on average response time because short requests spend on average less time waiting behing large requests.

### Streaming

Rather than perform a single, large blocking request to get all file data and then send data to the client all at once, we modified the file read procedure to instead perform blocking requests on a selectable (should be disk block size) capacity buffer and sending a buffer of information at a time. This actually slightly increases total time to send a file, but will allow the user to begin receiving data much sooner. Overall test time increased slightly with this feature.

### Caching

Caching is implemented with an LRU method. Files are served from the cached single-threaded because from-memory serving is fast enough that overhead incurred by multi threaded access to the cache is undesirable. After a file is read from disk, an explicit call is made to the cache to save that file for later retrieval. We elected not to implement a read-through cache to better support serving cached files from the mail listener thread while saving files in the cache from responder threads. Files served from the cache are passed by reference to avoid copying data for maximum performance.

### Parameter passing in URL

Many web services in the pre-AJAX era were purely about delivering static pages that could be refreshed in response to user activity in a synchronous way. Even now, with user facing JavaScript that asynchronously asks the server for data, the ability to interact with the server in a meaningful real time way is even more valuable.

This is the essence of a REST API, and our Zhtta server implements the GET capability. When a browser issues a GET request on a "dynamic" file on our server (ending in a .shtml extension) it is able to pass on all parameters occurring in a server side gashing.

We support the following syntax, embedded in the HTML file:

`<!--#exec cmd="cat $file | grep $keyword" -->`

Where the URL is something of the following:

`http://myzhtta_hostname:4414/get_keyword.shtml?file=myfile.txt&keyword=foo`

By enabling dynamic interactions that are scoped through the HTML we sent through the server, we are able to allow the user to pass us variables to serve without opening us up to arbitrary shell injection.

The way this works is by parsing the URL and detecting whether the question mark character is present by the parser. If the character is found in the URL, it is split on it and an iteration occurs on all arguments. Then we match up the argument name with any appearance of a `$` in the dynamic comments.

Once all the substitutions are made, it is passed on to gash just as before.

Our implementation is relatively safe from injection, because even though it supports spaces - gash does not allow for `&&`.

Please look at the www directory for examples of this in param_test.html

### Miscelenous improvements

Many of our miscellaneous improvements have been alluded to in earlier sections. We serve cached files from the main listener thread (since no blocking I/O is required) which saves us the overhead of spawning a thread and enqueue/dequeuing the request. This improved test performance by over 10%. 

## Performance

Because many of our changes were implemented concurrently, we do not have explicit benchmarking data for each change. The most explicit benchmarks we have taken were before caching was implemented (almost no performance changes to the given code) where the test run completed in 108 seconds on one of our machines. After caching, scheduling, and streaming were implemented, the test completes in roughly 22 seconds. With final performance improvements as well as tuning variable parameters (cache size, streaming block size, number of request threads) our runtime on that same hardware is down to approximately 17 seconds. Note that all of these tests are with freshly generated test data as well as with a newly spun up server. For almost any configuration (with caching), the server completes a second iterated test in 1.6 seconds because all files are just immediately served from the cache.

## Final Considerations

One trade-off decision faced was the cost of correctness in the cache. We elected to pay the cost in this case. Before a file is served from the cache, a blocking system call is made to check the modified date on the file and this is compared to the last modified date for the cached version (saved in the cache). If the on-disk version is newer, that request is not served from the cache. This requires an extra I/O operation before EVERY cache operation, a considerable cost. This decision would certainly depend on the application served. In an application that could tolerate some stale data, we would have implemented a time-to-live on cached data instead, bypassing the need for I/O to check staleness. Because the application was unknown, we elected for the safer, correct choice. We did not implement both versions of this choice to benchmark, but estimate the cost at more than double for cached files served and negligible for files not served from the cache.

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

