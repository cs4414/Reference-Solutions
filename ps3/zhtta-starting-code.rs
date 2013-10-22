//
// zhtta.rs
//
// Running on Rust 0.8
//
// Reference solution for PS3
// 
// Note: it would be very unwise to run this server on a machine that is
// on the Internet and contains any sensitive files!
//
// University of Virginia - cs4414 Fall 2013
// Weilin Xu and David Evans
// Version 0.1

extern mod extra;

use std::rt::io::*;
use std::rt::io::net::ip::{SocketAddr, Ipv4Addr};
use std::io::println;
use std::cell::Cell;
use std::{os, str, io};
use extra::arc;
use std::comm::*;

static PORT:    int = 4414;
static IPV4_LOOPBACK: &'static str = "127.0.0.1";
static mut visitor_count: uint = 0;

struct sched_msg {
    stream: Option<std::rt::io::net::tcp::TcpStream>,
    filepath: ~std::path::PosixPath
}

fn main() {
    let req_vec: ~[sched_msg] = ~[];
    let shared_req_vec = arc::RWArc::new(req_vec);
    let add_vec = shared_req_vec.clone();
    let take_vec = shared_req_vec.clone();
    
    let (port, chan) = stream();
    let chan = SharedChan::new(chan);
    
    // add file requests into queue.
    do spawn {
        while(true) {
            do add_vec.write |vec| {
                //println("add_vec");
                if (port.peek()) {
                    let tf:sched_msg = port.recv();
                    (*vec).push(tf);
                    println(fmt!("add to queue, size: %ud", (*vec).len()));
                }
            }
        }
    }
    
    // take file requests from queue, and send a response.
    // FIFO
    do spawn {
        while(true) {
            do take_vec.write |vec| {
                //println("take_vec");
                if ((*vec).len() > 0) {
                    let tf_opt: Option<sched_msg> = (*vec).shift_opt();
                    let mut tf = tf_opt.unwrap();
                    println(fmt!("pop from queue, size: %ud", (*vec).len()));
                    /*
                    println(fmt!("serve large file: "));
                    
                    let mut buf: ~[u8];
                    let buf_len: uint = 100*1024;
                    let mut file_reader = io::file_reader(tf.filepath).unwrap();
                    while true {
                        buf = file_reader.read_bytes(buf_len);
                        if (!buf.is_empty()) {
                            tf.stream.write(buf);
                        } else { break;}
                    }*/
                    
            

                    match io::read_whole_file(tf.filepath) { // killed if file size is larger than memory size.
                        Ok(file_data) => {
                            println(fmt!("begin serving file [%?]", tf.filepath));
                            tf.stream.write(file_data);
                            println(fmt!("finish file [%?]", tf.filepath));
                        }
                        Err(err) => {
                            println(err);
                        }
                    } 
                }
            }
        }
    }
    
    let socket = net::tcp::TcpListener::bind(SocketAddr {ip: Ipv4Addr(0,0,0,0), port: PORT as u16});
    
    println(fmt!("Listening on tcp port %d ...", PORT));
    let mut acceptor = socket.listen().unwrap();
    
    // we can limit the incoming connection count.
    //for stream in acceptor.incoming().take(10 as uint) {
    for stream in acceptor.incoming() {
        let stream = Cell::new(stream);
        
        // Start a new task to handle the connection
        let child_chan = chan.clone();
        do spawn {
            unsafe {
                visitor_count += 1;
            }
            
            let mut stream = stream.take();
            let mut buf = [0, ..500];
            stream.read(buf);
            let request_str = str::from_utf8(buf);
            
            let req_group : ~[&str]= request_str.splitn_iter(' ', 3).collect();
            if req_group.len() > 2 {
                let path = req_group[1];
                println(fmt!("Request for path: \n%?", path));
                
                let file_path = ~os::getcwd().push(path.replace("/../", ""));
                if !os::path_exists(file_path) || os::path_is_dir(file_path) {
                    println(fmt!("Request received:\n%s", request_str));
                    let response: ~str = fmt!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n
                         <doctype !html><html><head><title>Hello, Rust!</title>
                         <style>body { background-color: #111; color: #FFEEAA }
                                h1 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm red}
                                h2 { font-size:2cm; text-align: center; color: black; text-shadow: 0 0 4mm green}
                         </style></head>
                         <body>
                         <h1>Greetings, Krusty!</h1>
                         <h2>Visitor count: %u</h2>
                         </body></html>\r\n", unsafe{visitor_count});

                    stream.write(response.as_bytes());
                }
                else {
                    // may do scheduling here
                    let msg: sched_msg = sched_msg{stream: stream, filepath: file_path.clone()};
                    child_chan.send(msg);
                    
                    
                    println(fmt!("get file request: %?", file_path));
                }
            }
            println!("connection terminates")
        }
    }
}
