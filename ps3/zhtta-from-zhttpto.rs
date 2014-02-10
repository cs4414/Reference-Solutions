//
// zhtta.rs
//
// Starting code for PS3
// Running on Rust 0.9
//
// Note that this code has serious security risks!  You should not run it 
// on any system with access to sensitive files.
// 
// University of Virginia - cs4414 Spring 2014
// Weilin Xu and David Evans
// Version 0.4
#[feature(globs)];
extern mod extra;

use std::io::*;
use std::io::net::ip::{SocketAddr};
use std::{os, str};
use std::hashmap::HashMap;

use extra::arc::MutexArc;

static IP: &'static str = "127.0.0.1";
static PORT:        int = 4414;
static mut visitor_count: uint = 0;

struct HTTP_Request {
    priority: uint,
    //stream: Option<std::io::net::tcp::TcpStream>,
    peer_name: ~str, // as a key to TcpStream
    path: ~std::path::PosixPath
}

impl Ord for HTTP_Request {
    fn lt(&self, other: &HTTP_Request) -> bool {
        if self.priority > other.priority { true } else { false }
    }
}


fn main() {
    let req_queue: ~[HTTP_Request] = ~[]; // Be used as FIFO queue.
    let shared_req_queue = MutexArc::new(req_queue);
    
    let stream_map: HashMap<~str, Option<std::io::net::tcp::TcpStream>> = HashMap::new();
    let shared_stream_map = MutexArc::new(stream_map);
    
    // Create a task of request responder.
    let (notify_port, shared_notify_chan) = SharedChan::new();
    
    let req_queue_get = shared_req_queue.clone();
    let stream_map_get = shared_stream_map.clone();
    do spawn {
        let (request_port, request_chan) = Chan::new();
        let (stream_port, stream_chan) = Chan::new();
        loop {
            notify_port.recv();
            
            req_queue_get.access( |req_queue| {
                match req_queue.shift_opt() {
                    None => { /* do nothing */ }
                    Some(req) => {
                        request_chan.send(req);
                    }            
                }
            });
            
            let request = request_port.recv();
            //println(format!("serve file: {:?}", request.path));
            
            // Get stream from hashmap.
            unsafe {
                stream_map_get.unsafe_access(|local_stream_map| {
                    let stream = local_stream_map.pop(&request.peer_name).expect("no option tcpstream");
                    stream_chan.send(stream);
                });
            }
            let mut stream = stream_port.recv();
                        
            // Respond with file content.
            let contents = File::open(request.path).read_to_end();
            stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n".as_bytes());
            stream.write(contents);
        }
    }

    // Create socket.
    let addr = from_str::<SocketAddr>(format!("{:s}:{:d}", IP, PORT)).expect("Address error.");
    let mut acceptor = net::tcp::TcpListener::bind(addr).listen();
    
    println(format!("Listening on [{:s}] ...", addr.to_str()));
    
    for stream in acceptor.incoming() {
        let (queue_port, queue_chan) = Chan::new();
        queue_chan.send(shared_req_queue.clone());
        
        let notify_chan = shared_notify_chan.clone();
        let stream_map_arc = shared_stream_map.clone();
        // Spawn a task to handle the connection
        do spawn {
            unsafe {
                visitor_count += 1;
            }
            
            let shared_req_queue = queue_port.recv();
          
            let mut stream = stream;
            
            let (pn_port, pn_chan) = Chan::new();
            
            match stream {
                Some(ref mut s) => {
                             match s.peer_name() {
                                Some(pn) => {pn_chan.send(pn.to_str()); println(format!("Received connection from: [{:s}]", pn.to_str()));},
                                None => ()
                             }
                           },
                None => ()
            }
            
            
            let peer_name = pn_port.recv();
            
            let mut buf = [0, ..500];
            stream.read(buf);
            let request_str = str::from_utf8(buf);
            println(format!("Received request:\n{:s}", request_str));
            
            let req_group : ~[&str]= request_str.splitn(' ', 3).collect();
            if req_group.len() > 2 {
                let path_str = "." + req_group[1].to_owned();
                println(format!("Request for path: \n{:?}", path_str));
                
                let mut path_obj = ~os::getcwd();
                path_obj.push(path_str.clone());
                
                if !path_obj.exists() || path_obj.is_dir() {
                    let response: ~str = 
                        format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
                         <doctype !html><html><head><title>Hello, Rust!</title>
                         <style>body \\{ background-color: \\#111; color: \\#FFEEAA \\}
                                h1 \\{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red\\}
                                h2 \\{ font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green\\}
                         </style></head>
                         <body>
                         <h1>Greetings, Krusty!</h1>
                         <h2>Visitor count: {0:u}</h2>
                         </body></html>\r\n", unsafe{visitor_count});
                    stream.write(response.as_bytes());
                } else {
                    // request scheduling
                    println("request scheduling begins.");
                    
                    // Save stream in hashmap for later response.
                    let (stream_port, stream_chan) = Chan::new();
                    stream_chan.send(stream);
                    println("send to unsafe.");
                    unsafe {
                        stream_map_arc.unsafe_access(|local_stream_map| {
                            let stream = stream_port.recv();
                            local_stream_map.swap(peer_name.clone(), stream);
                        });
                    }
                    
                    // Enqueue the HTTP request.
                    let req = HTTP_Request{priority: 1, peer_name: peer_name.clone(), path: path_obj.clone()};
                    
                    let (req_port, req_chan) = Chan::new();
                    req_chan.send(req);
                    shared_req_queue.access(|local_req_queue| {
                        let req: HTTP_Request = req_port.recv();
                        local_req_queue.push(req);
                    });
                        
                    notify_chan.send(()); // Send incoming notification to responder.
                    println("request enqueued.");
                }
            }
            println!("connection terminates")
        }
    }
}
