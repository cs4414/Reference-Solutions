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
use extra::priority_queue::PriorityQueue;

static IP: &'static str = "127.0.0.1";
static PORT:        uint = 4414;
static mut visitor_count: uint = 0;

struct HTTP_Request {
     // Use peer_name as the key to TcpStream. 
     // Due to a bug in extra::arc in Rust 0.9, it is very inconvenient to use TcpStream without the "Freeze" bound.
     // Issue: https://github.com/mozilla/rust/issues/12139 
    peer_name: ~str,
    path: ~std::path::PosixPath,
    file_size: uint,
    priority: uint,
}

impl Ord for HTTP_Request {
    fn lt(&self, other: &HTTP_Request) -> bool {
        if self.priority > other.priority { true } else { false }
    }
}

struct WebServer {
    ip: ~str,
    port: uint,
    working_directory: ~str,
    
    //request_queue: PriorityQueue<HTTP_Request>,
    request_queue_arc: MutexArc<PriorityQueue<HTTP_Request>>,
    
    //stream_map: HashMap<~str, Option<std::io::net::tcp::TcpStream>>,
    stream_map_arc: MutexArc<HashMap<~str, Option<std::io::net::tcp::TcpStream>>>,
    
    notify_port: Port<()>,
    shared_notify_chan: SharedChan<()>,
}

impl WebServer {
    fn new(ip: &str, port: uint, working_directory: &str) -> WebServer {
        // change directory to working_directory
        // chroot jain in working_directory
        let (notify_port, shared_notify_chan) = SharedChan::new();
        WebServer {
            ip: ip.to_owned(),
            port: port,
            working_directory: working_directory.to_owned(),
            
            //request_queue: PriorityQueue::new(),
            request_queue_arc: MutexArc::new(PriorityQueue::new()),
            
            //stream_map: HashMap::new(),
            stream_map_arc: MutexArc::new(HashMap::new()),
            
            notify_port: notify_port,
            shared_notify_chan: shared_notify_chan,
        }
    }
    
    fn listen(&mut self) {
        // Create socket.
        let addr = from_str::<SocketAddr>(format!("{:s}:{:u}", self.ip, self.port)).expect("Address error.");
        
        
        let request_queue_arc = self.request_queue_arc.clone();
        let shared_notify_chan = self.shared_notify_chan.clone();
        let stream_map_arc = self.stream_map_arc.clone();
        
        do spawn {
            let mut acceptor = net::tcp::TcpListener::bind(addr).listen();
            println(format!("Listening on [{:s}] ...", addr.to_str()));
            
            for stream in acceptor.incoming() {
                let (queue_port, queue_chan) = Chan::new();
                queue_chan.send(request_queue_arc.clone());
                
                let notify_chan = shared_notify_chan.clone();
                let stream_map_arc = stream_map_arc.clone();
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
                                        Some(pn) => {pn_chan.send(pn.to_str()); debug!("=====Received connection from: [{:s}]=====", pn.to_str());},
                                        None => ()
                                     }
                                   },
                        None => ()
                    }
                    
                    
                    let peer_name = pn_port.recv();
                    
                    let mut buf = [0, ..500];
                    stream.read(buf);
                    let request_str = str::from_utf8(buf);
                    debug!("Request :\n{:s}", request_str);
                    
                    let req_group : ~[&str]= request_str.splitn(' ', 3).collect();
                    if req_group.len() > 2 {
                        let path_str = "." + req_group[1].to_owned();
                        
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
                            
                            // Save stream in hashmap for later response.
                            let (stream_port, stream_chan) = Chan::new();
                            stream_chan.send(stream);
                            unsafe {
                                // Use unsafe method, because TcpStream in Rust 0.9 doesn't have "Freeze" bound.
                                stream_map_arc.unsafe_access(|local_stream_map| {
                                    let stream = stream_port.recv();
                                    local_stream_map.swap(peer_name.clone(), stream);
                                });
                            }
                            
                            // Get file size.
                            let file_size = std::io::fs::stat(path_obj).size as uint;
                            
                            // Enqueue the HTTP request.
                            let req = HTTP_Request{peer_name: peer_name.clone(), path: path_obj.clone(), file_size: file_size, priority: file_size};
                            
                            let (req_port, req_chan) = Chan::new();
                            req_chan.send(req);
                            debug!("Waiting for queue mutex.");
                            shared_req_queue.access(|local_req_queue| {
                                debug!("Got queue mutex lock.");
                                let req: HTTP_Request = req_port.recv();
                                local_req_queue.push(req);
                                // To see debug! outputs set the RUST_LOG environment variable, e.g.: export RUST_LOG="zhtta-from-zhttpto=debug" 
                                debug!("A new request enqueued, now the length of queue is {:u}.", local_req_queue.len());
                            });
                            
                            notify_chan.send(()); // Send incoming notification to responder.
                        }
                    }
                    //debug!("=====Terminated connection from [{:s}].=====", peer_name);
                }
            } // for
        }
    }
    
    fn run(&mut self) {
        let req_queue_get = self.request_queue_arc.clone();
        let stream_map_get = self.stream_map_arc.clone();
        // I couldn't send port into another task. So I have to make it as the main task that can access self.notify_port.
        //let notify_port = self.notify_port;
        
        let (request_port, request_chan) = Chan::new();
        let (stream_port, stream_chan) = Chan::new();
        loop {
            self.notify_port.recv();
            
            req_queue_get.access( |req_queue| {
                match req_queue.maybe_pop() { // SRPT queue.
                    None => { /* do nothing */ }
                    Some(req) => {
                        request_chan.send(req);
                        debug!("A new request dequeued, now the length of queue is {:u}.", req_queue.len());
                    }            
                }
            });
            
            let request = request_port.recv();
            //println(format!("serve file: {:?}", request.path));
            
            // Get stream from hashmap.
            // Use unsafe method, because TcpStream in Rust 0.9 doesn't have "Freeze" bound.
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
            // Close stream automatically.
            debug!("=====Terminated connection from [{:s}].=====", request.peer_name);
        }
    }
}

fn main() {
    let mut zhtta = WebServer::new(IP, PORT, "./");
    zhtta.listen();
    zhtta.run();

    return;
}