//
// zhttpto.rs
//
// Reference solution for PS1
// Running on Rust 0.9
//
// Note that this code has serious security risks!  You should not run it 
// on any system with access to sensitive files.
//
// Special thanks to Kiet Tran for providing code we incorporated into this.
// 
// University of Virginia - cs4414 Fall 2013
// Weilin Xu and David Evans
// Version 0.3
#[feature(globs)];
use std::io::*;
use std::io::net::ip::{SocketAddr};
use std::{os, str};

static IP: &'static str = "127.0.0.1";
static PORT:        int = 4414;
static mut visitor_count: uint = 0;

fn main() {
    let addr = from_str::<SocketAddr>(format!("{:s}:{:d}", IP, PORT)).expect("Address error.");
    let mut acceptor = net::tcp::TcpListener::bind(addr).listen();
    
    println(format!("Listening on [{:s}] ...", addr.to_str()));
    
    for stream in acceptor.incoming() {
        // Spawn a task to handle the connection
        do spawn {
            unsafe {
                visitor_count += 1;
            }
            let mut stream = stream;
            
            match stream {
                Some(ref mut s) => {
                             match s.peer_name() {
                                Some(pn) => {println(format!("Received connection from: [{:s}]", pn.to_str()));},
                                None => ()
                             }
                           },
                None => ()
            }
            
            let mut buf = [0, ..500];
            stream.read(buf);
            let request_str = str::from_utf8(buf);
            
            let req_group : ~[&str]= request_str.splitn(' ', 3).collect();
            if req_group.len() > 2 {
                let path_str = req_group[1];
                println(format!("Request for path: \n{:?}", path_str));
                
                let mut path_obj = os::getcwd();
                path_obj.push("."+path_str);
                if !path_obj.exists() || path_obj.is_dir() {
                    println(format!("Received request:\n{:s}", request_str));
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
                }
                else {
                    println(format!("serve file: {:s}", path_str));

                    let contents = File::open(&path_obj).read_to_end();
                    stream.write(contents);
                }
            }
            println!("connection terminates")
        }
    }
}
