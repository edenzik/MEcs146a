Title: Programming Assignment 1 Answers

Author: Eden Zik

A web server is a program that responds to HTTP requests. This zhttpto is a small Rust web server that receives requests from a client, prints that the request occurred to the console, and responds to the client with a string of HTML that can then be viewed using a web browser.

1. Problem 1

	The program prints to the console whenever a request is received. The body of a standard GET request includes the User-Agent, which is the platform from which the request was sent.

	The user agent reported by the server is the following:

	```
Mozilla/5.0 (Macintosh; Intel Mac OS X 10.10; rv:36.0) Gecko/20100101 Firefox/36.0
```

	The user agent consists of the system used by the user to access the web server. In my case, the Mozilla 5.0 system is a legacy indicator from the Netscape days. However, the "Macintosh Intel Mac OS X 10.10; rv:36.0" is the version of the operating system I am using. Gecko is the layout engine for Firefox, and Firefox/36.0 is the web browser followed by its version number.

2. Problem 2

	The zhttpto works by dispatching a thread for every request received, and responding to it independently. In order for each thread to show the request number overall, there needs to be a way for all threads to increment and print a global value, "visitor_count", on each request.

	The way in which I added a visitor count was by making it a static global variable above the main function, as follows:
	
	```
static mut visitor_count: int = 0;
```

	I then modified the Thread spawning block to print the current visitor count and increment it, as follows:

	```
Thread::spawn(move|| {
	match stream.peer_name() {
		Err(_) => (),
		Ok(pn) => {
			println!("Received connection from: [{}], 			Count - {}", pn, visitor_count);
			visitor_count+=1;
	}
}
```

	When compiling this code, the Rust compiler complains  `use of mutable static requires unsafe function or block`. This indicates that the global mutable static variable I placed at the top of the main method is unsafe - presumably due to the potential for concurrent modification by several running threads responding to requests. This type of data race could lead the counter to be modified concurrently, as nothing about the variable guarantees its incrementation is atomic. A classic case of this is the following:
	- `visitor_count = 0`
	- Request received. Thread 1 spawned.
	- Request recieved. Thread 2 spawned.
	- Thread 1 copies value of visitor_count (`=0`) to CPU for incrementing.
	- Thread 2 copies value of visitor_count (`=0`) to CPU for incrementing.
	- Thread 1 writes the new value of visitor_count (`=1`) back to memory.
	- Thread 2 writes the new value of visitor_count (`=1`) back to memory.
	- Value of `visitor_count = 1`, should be `=2`.

    There are multiple ways to address this problem. The worst one would be effectively ignoring the danger, by wrapping the access in so called `unsafe` block, which tells the compiler to ignore the danger posed by potential concurrent access. This can be done by the following:
    
	```
Thread::spawn(move|| {
	match stream.peer_name() {
		Err(_) => (),
		Ok(pn) => {
			unsafe {
				println!("Received connection from: [{}], 				Requests - {}", pn, visitor_count);
				visitor_count+=1;
			}
	}
}
```

	The counter is then incremented on each request. A potential safe alterantive I've thought about is to create a local mutable variable in main, and then increment it in each thread as follows:
	
	```
fn main() {
    let addr = "127.0.0.1:4414";

    let mut acceptor = TcpListener::bind(addr).unwrap().listen().unwrap();

    println!("Listening on [{}] ...", addr);

    let mut visitor_count = 0;                  //Local mutable variable
...
```

	This did not work - as each thread apperaed to have an independnet value of `visitor_count`. This left me with the assumption that the variable was passed by value to each thread when it was dispatched. In lack of more documentation, I assumed the other safe way to do it is to increment the `visitor_count` variable when iteration over requests occur, prior to the dispatching of the thread. The following is the way this is done:

	```
for stream in acceptor.incoming() {
        match stream {
            Err(_) => (),
            Ok(mut stream) => {
                visitor_count += 1;             //Increments variable
                // Spawn a thread to handle the connection
                Thread::spawn(move|| {
                    match stream.peer_name() {
                        Err(_) => (),
                        Ok(pn) => {
                            println!("Received connection from: [{}] - Requests - {}", pn, visitor_count);
                        }
                    }
```

	This code exists in the "zhttpto_q2_safe.rs" file. Because it is much better, it will be used for the remainder of this assignment as the superior version of incrementing count.

3. Problem 3

	Incrementing the value on each access only requires keeping track of a mutable variable. Another, more complicated part, is returning a response to the user that includes that number. Using the `format` construct in Rust, I was able to create a response that includes a `visitor_count`. This was done as follows:

	```
let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
                         <doctype !html><html><head><title>Hello, Rust!</title>
                         <style>body {{ background-color: #111; color: #FFEEAA }}
                                h1 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red}}
                                h2 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green}}
                         </style></head>
                         <body>
                         <h1>Greetings, Krusty! {}</h1>
                         </body></html>\r\n", visitor_count);
```

	The response is now `Hello, Krusty` followed by the request number.
	
4. Problem 4

	Even with the counter, this web server is not very useful. We would like to modify our server to return static pages.
	
	I used a split method to take apart a request by space, and iterated over the elements in it to get to the second whitespace serprated element, according to the GET /<path> HTTP/1.1 where <path> is a non-empty file system path. The server then sends out a response when it reads the file. If the request fails, the server responds with the default response including the counter.
	
	This was a more sophisticated parsing question - because Rust makes it a bit difficult. Having to account for all cases is important, made necessary by the match statement.
	
	```
	 match File::open(&path).read_to_string() {
                                Ok(string) => {
                                    stream.write(string.as_bytes());
                                }
                                Err(_) => {
                                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
                         <doctype !html><html><head><title>Hello, Rust!</title>
                         <style>body {{ background-color: #111; color: #FFEEAA }}
                                h1 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red}}
                                h2 {{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green}}
                         </style></head>
                         <body>
                         <h1>Greetings, Krusty! {}</h1>
                         </body></html>\r\n", visitor_count);
                                    stream.write(response.as_bytes());
                                }
                            };
	```
	
5. Problem 5

	This problem required some major code refactoring - and hence serious changes were made to the code in order for it to conform better to Rust standards. In this case, I parsed the file name to get the extension, and only let the user access HTML files. If the file does not fit this, a permission denied error is returned. Please refer to the code for this, as the modifications are too long to place here.
	
	Another neat addition is the "File Not Found" response I chose to add - all using Rust match statements.
	
	I have done my best to follow the best Rust convetions, including matches and type safety.
	




 
   








