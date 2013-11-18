//
// zhtta.rs
//
// Running on Rust 0.8
//
// TODO - Towards PS3: application-layer file caching
// Towards PS3: SPT scheduling
// Towards PS3: improving concurrency 
// Towards PS3: eliminating long-blocked IO
// 
// Note: it would be very unwise to run this server on a machine that is
// on the Internet and contains any sensitive files!
//
// University of Virginia - cs4414 Fall 2013
// Weilin Xu and David Evans

extern mod extra;

use std::rt::io::*;
use std::rt::io::net::ip::SocketAddr;
use std::io::println;
use std::cell::Cell;
use std::{os, str, path};
use extra::arc;
use std::comm::*;
use extra::getopts::*;
use std::vec;
use extra::priority_queue::PriorityQueue;

static PORT:    int = 4414;
static IP: &'static str = "127.0.0.1";
static mut visitor_count: uint = 0;
static FILE_CHUNK_BUF_SIZE: int = 512000; // default size of buffer (bytes)
static RESPONDER_CONCURRENCY: int = 5; // default amount of concurrent tasks

struct sched_msg {
    stream: Option<std::rt::io::net::tcp::TcpStream>,
    file_path: ~std::path::PosixPath,
    file_size: uint,
    priority: uint
}

impl Ord for sched_msg {
    fn lt(&self, other: &sched_msg) -> bool {
        if self.priority > other.priority { true } else { false }
    }
}

struct Scheduler{
    pqueue: PriorityQueue<sched_msg>
}

impl Scheduler {
    fn new() -> Scheduler { 
        Scheduler {
            pqueue: PriorityQueue::new()
        }
    }
    
    fn is_wahoo(&mut self, ip_s: &str) -> bool {
        if (ip_s.starts_with("128.143.") || ip_s.starts_with("137.54.") || ip_s.starts_with("172.26.") 
                                         || ip_s.starts_with("50.134.")) {
            return true;
        } else {
            return false;
        }
    }

    fn add_sched_msg(&mut self, mut sm: sched_msg) {
        // A file with size smaller than 40 KByte should be responsed quickly 
        let mut priority = sm.file_size as uint / 20480;

        let ip_s = match sm.stream {
            Some(ref mut stream) => {
                match stream.peer_name() {
                    Some(pn) => pn.ip.to_str(),
                    None => "0".to_owned()
                }
            }
            None => "0".to_owned()
        };
        
        // Wahoo-First scheduling
        if (self.is_wahoo(ip_s)) {
            priority = (priority as f32 * 0.6) as uint;
            sm.priority = priority;
        }
        
        //println(fmt!("size: %u, priority: %u", file_size as uint, priority as uint));
        self.pqueue.push(sm);
    }
}

fn print_usage(program: &str, _opts: &[Opt]) {
    printfln!("Usage: %s [options]", program);
    println("--ip          \tIP");
    println("--port        \tPORT");
    println("--bufsize     \tFILE_CHUNK_BUF_SIZE");
    println("--concurrency \tRESPONDER_CONCURRENCY");
    println("--dir         \tWWW_DIR");
    println("-h --help     \tUsage");
}

fn get_arg_int_by_key(matches: &Matches, key: &str, default: int) -> int {
    let value : int = match matches.opt_str(key) {
        Some(p) => { match from_str(p) { Some(i) => {i}, None => {default} } }
        None() => { default } 
    };
    return value;
}

fn main() {
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
    let program = args[0].clone();
    
    let opts = ~[
        optopt("ip"),
        optopt("port"),
        optopt("bufsize"),
        optopt("concurrency"),
        optopt("dir"),
        optflag("h"),
        optflag("help")
    ];
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };
    if matches.opt_present("h") || matches.opt_present("help") {
        print_usage(program, opts);
        return;
    }
    
    let ip_str = match matches.opt_str("ip") {Some(ip) => {ip}, None => {IP.to_owned()}};
    let port_int = get_arg_int_by_key(&matches, "port", PORT);
    let bufsize = get_arg_int_by_key(&matches, "bufsize", FILE_CHUNK_BUF_SIZE);
    let concurrency = get_arg_int_by_key(&matches, "concurrency", RESPONDER_CONCURRENCY);
    let www_dir = match matches.opt_str("dir") {Some(dir) => {dir}, None => {"./".to_owned()}};
    
    println("\nStarting zhtta...");
    printfln!("Size of file chunk buffer: %d bytes", bufsize);
    printfln!("Number of concurrent responders: %?", concurrency);
    printfln!("Serving at %s", path::PosixPath(www_dir).to_str());
    
    /* Finish processing program arguments and initiate the parameters. */
    
    os::change_dir(&path::PosixPath(www_dir));

    let sched = Scheduler::new();
    let shared_sched = arc::RWArc::new(sched);
    
    let (port, chan) = stream();
    let chan = SharedChan::new(chan);
    let port = SharedPort::new(port);
    
    // dequeue file requests, and send responses.
    // SPT
    for _ in range(0, concurrency) {
        let child_shared_sched = shared_sched.clone();
        let port = port.clone();
        do spawn {
            let (sm_port, sm_chan) = stream();
            let mut file_chunk_buf: ~[u8] = vec::with_capacity(bufsize as uint);
            unsafe {vec::raw::set_len(&mut file_chunk_buf, bufsize as uint);} // File_reader.read() doesn't recognize capacity, but len() instead. A wrong design?
            
            loop {
                port.recv(); // wait for arrving notification
                do child_shared_sched.write |sched| {
                    match sched.pqueue.maybe_pop() {
                        None => { /* do nothing */ }
                        Some(msg) => {sm_chan.send(msg);}
                    }
                }
                let mut tf: sched_msg = sm_port.recv(); // wait for the dequeued request to handle
                // Print the serving file's name.
                printfln!("%s", tf.file_path.components[tf.file_path.components.len()-1]);
                let mut file_reader = file::open(tf.file_path, Open, Read).unwrap();
                tf.stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n".as_bytes());
                while (!file_reader.eof()) {
                    match file_reader.read(file_chunk_buf) {
                        Some(len) => {tf.stream.write(file_chunk_buf.slice(0, len));}
                        None => {}
                    }
                }
            }
        }
    }

    let ip = match FromStr::from_str(ip_str) { Some(ip) => ip, 
                                           None => { println(fmt!("Error: Invalid IP address <%s>", IP));
                                                     return;},
                                         };
                                         
    let socket = net::tcp::TcpListener::bind(SocketAddr {ip: ip, port: port_int as u16});
    let mut acceptor = socket.listen();
    
    println(fmt!("Listening on %s:%d ...", ip_str, port_int));
    
    for stream in acceptor.incoming() {
        let stream = Cell::new(stream);
        //println(fmt!("new stream: %?", stream));
        // Start a new task to handle the each connection
        let child_chan = chan.clone();
        let child_shared_sched = shared_sched.clone();
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
                //println(fmt!("Request for path: \n%?", path));
                
                let file_path = ~os::getcwd().push(path.replace("/../", ""));
                if !os::path_exists(file_path) || os::path_is_dir(file_path) {
                    //println(fmt!("Request received:\n%s", request_str));
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
                    // Requests scheduling
                    let file_size = match std::rt::io::file::stat(file_path) {
                                        Some(s) => s.size as uint,
                                        None() => 0,
                    };
                    let msg: sched_msg = sched_msg{priority: 0, stream: stream, file_path: file_path.clone(), file_size: file_size};
                    let (sm_port, sm_chan) = std::comm::stream();
                    sm_chan.send(msg);
                    
                    do child_shared_sched.write |sched| {
                        let msg = sm_port.recv();
                        sched.add_sched_msg(msg);
                        //println("new request added to queue");
                    }
                    child_chan.send(""); //notify the new request in queue.
                }
            }
            //println!("connection terminates")
        }
    }
}
