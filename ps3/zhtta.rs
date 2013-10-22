//
// zhtta.rs
//
// Running on Rust 0.8
//
// Starting code for PS3
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
use std::comm::*;
use std::cmp::Ord;
use extra::arc;
use extra::priority_queue::PriorityQueue;
use std::task;
use extra::sync;

static PORT:    int = 4414;
static IPV4_LOOPBACK: IpAddr = Ipv4Addr(0,0,0,0);
static visitor_count: uint = 0u;

static LEVEL1: u64 = 1024000;//1MB
static LEVEL2: u64 = 15360000;//15MB
static LEVEL3: u64 = 25600000;//25MB
static LEVEL4: u64 = 56320000;//55MB

struct sched_msg {
    priority: uint,
    stream: Option<std::rt::io::net::tcp::TcpStream>,
    file_path: ~std::path::PosixPath
}

impl Ord for sched_msg {
    fn lt(&self, other: &sched_msg) -> bool {
        if self.priority > other.priority { true } else { false }
    }
}

struct Scheduler(PriorityQueue<sched_msg>);

impl Scheduler {
    fn new() -> Scheduler { 
        Scheduler(PriorityQueue::new())
    }

    fn add_sched_msg(&mut self, mut sm: sched_msg) {
        let file_size = match file::open(sm.file_path, Open, Read) {
            Some(filestream) => {
                let mut fs = filestream;
                fs.seek(0, SeekEnd);
                fs.tell()
            }
            None => 0
        };
        let mut priority = if file_size < LEVEL1 { 10 } 
                           else if file_size < LEVEL2 { 20 }
                           else if file_size < LEVEL3 { 30 }
                           else if file_size < LEVEL4 { 40 }
                           else { 50 };

        let ip_s = match sm.stream {
            Some(ref mut stream) => {
                match stream.peer_name() {
                    Some(pn) => pn.ip.to_str(),
                    None => "0".to_owned()
                }
            }
            None => "0".to_owned()
        };
        if !(ip_s.starts_with("128.143.") || ip_s.starts_with("137.54.")
                                          || ip_s.starts_with("50.134.")) {
            priority += 5;
        }
        sm.priority = priority;
        self.push(sm);
    }



    fn do_task(&mut self) {
        match self.maybe_pop() {
            None => { /* do nothing */ }
            Some(sm) => {
                let mut sm = sm;
                match io::read_whole_file(sm.file_path) {
                    Ok(file_data) => {
                        println(fmt!("begin serving file [%?]", sm.file_path));
                        sm.stream.write(file_data);
                        //do task::spawn_with(sm) |mut sm: sched_msg| {sm.stream.write("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n".as_bytes());}
                    }
                    Err(err) => {
                        println(err);
                    }
                }
            }
        }
    }
}

fn main() {
    let sched = Scheduler::new();
    let add_sched = arc::RWArc::new(sched);
    let do_sched = add_sched.clone();

    let shared_v_count = arc::RWArc::new(visitor_count);
    
    let (port, chan) = stream();
    let chan = SharedChan::new(chan);
    
    // add file requests into queue.
    do spawn {
        let (sm_port, sm_chan) = stream();
        loop {
            //println("wait for sm from chan.");
            let sm: sched_msg = port.recv();
            //println("get sm from port.");
            sm_chan.send(sm);
            do add_sched.write |sched| {
                let sm = sm_port.recv();
                sched.add_sched_msg(sm);
                println("add to queue");
                while (port.peek()) {
                    let sm: sched_msg = port.recv();
                    sched.add_sched_msg(sm);
                    println("add more to queue");
                }
            }
        }
    }


    
    // take file requests from queue, and send a response.
    // take response
    // unknown function in the scope will block the whole thread, so use a new scheduler to create this task.
    do task::spawn_sched(task::SingleThreaded) {
        let (sm_port, sm_chan) = stream();
        loop {
            //println("lock for getting sm");
            do do_sched.write |sched| {
                match sched.maybe_pop() {
                    None => { /* do nothing */ }
                    Some(sm) => {sm_chan.send(sm);}
                }
            }
            //println("unlock for getting sm");
            
            if (sm_port.peek()) {
                //println("get sm from queue");
                let mut sm = sm_port.recv();
                match io::read_whole_file(sm.file_path) {
                    Ok(file_data) => {
                        println(fmt!("begin serving file [%?]", sm.file_path));
                        //sm.stream.write("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n".as_bytes());
                        sm.stream.write(file_data);
                        println("finish serving");
                    }
                    Err(err) => {
                        println(err);
                    }
                }
            } else {
                //println("no sm at all");
            }
        }
    }
    
    let socket = net::tcp::TcpListener::bind(SocketAddr {ip: IPV4_LOOPBACK, port: PORT as u16});
    
    println!("Listening on tcp port {} ...", PORT);
    let mut acceptor = socket.listen().unwrap();
    
    // we can limit the incoming connection count.
    //for stream in acceptor.incoming().take(10 as uint) {
    for stream in acceptor.incoming() {
        let stream = Cell::new(stream);
        
        // Start a new task to handle the connection
        let child_chan = chan.clone();
        let inc_v_count = shared_v_count.clone();
        do spawn {
            do inc_v_count.write |v_count| {
                *v_count += 1;
            }  
            
            let mut stream = stream.take();
            let mut buf = [0, ..500];
            stream.read(buf);
            let request_str = str::from_utf8(buf);
            
            let req_group : ~[&str]= request_str.splitn_iter(' ', 3).collect();
            if req_group.len() > 2 {
                let path = req_group[1];
                //println!("Request for path: \n{}", path);
                
                let file_path = ~os::getcwd().push(path.replace("/../", ""));
                if !os::path_exists(file_path) || os::path_is_dir(file_path) {
                    //println!("Request received:\n{}", request_str);
                    do inc_v_count.read |&v_count| {
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
                             </body></html>\r\n", v_count);

                        stream.write(response.as_bytes());
                    }
                }
                else {
                    // may do scheduling here
                    //println!("get file request: {:?}", file_path);
                    let msg: sched_msg = sched_msg{priority: 0, stream: stream, file_path: file_path.clone()};
                    child_chan.send(msg);
                    
                }
            }
            //println!("connection terminates")
        }
    }
}
