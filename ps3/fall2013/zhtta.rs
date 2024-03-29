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
use std::rt::io::net::ip::{SocketAddr};
use std::io::println;
use std::cell::Cell;
use std::{os, str, io};
use std::io::ReaderUtil;
use std::comm::*;
use std::cmp::Ord;
use extra::arc;
use extra::priority_queue::PriorityQueue;

use lru_cache::LRUCache;
use std::hashmap::HashMap;

mod gash;
mod lru_cache;

static PORT:    int = 4414;
static IP: &'static str = "127.0.0.1";
static visitor_count: uint = 0u;
// The number of concurrent response tasks.
static CONCURRENT_RESPONSESOR: int = 5;
// The file chunk size for reading.
static FILE_CHUNK_SIZE: int = 102400;
// Byte size of LRU cache
static CACHE_SIZE: uint = 256000000;

struct sched_msg {
    priority: uint,
    stream: Option<std::rt::io::net::tcp::TcpStream>,
    file_path: ~std::path::PosixPath,
    file_size: uint
}

impl Ord for sched_msg {
    fn lt(&self, other: &sched_msg) -> bool {
        if self.priority > other.priority { true } else { false }
    }
}

struct CacheManager{
    cache: LRUCache<~str, ~[u8]>,
    modified_map: HashMap<~str, u64>
}

impl CacheManager {
    fn new() -> CacheManager { 
        CacheManager {
            cache: LRUCache::new(CACHE_SIZE),
            modified_map: HashMap::new(),
        }
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
        if (is_wahoo(ip_s)) {
            priority = (priority as f32 * 0.6) as uint;
            sm.priority = priority;
        }
        
        //println(fmt!("size: %u, priority: %u", file_size as uint, priority as uint));
        self.pqueue.push(sm);
    }
}

fn is_wahoo(ip_s: &str) -> bool {
    if (ip_s.starts_with("128.143.") || ip_s.starts_with("137.54.") || ip_s.starts_with("172.26.") 
                                     || ip_s.starts_with("50.134.")) {
        return true;
    } else {
        return false;
    }
}

fn main() {
    let sched = Scheduler::new();
    let add_sched = arc::RWArc::new(sched);
    let do_sched = add_sched.clone();
    
    let cache_mgr = CacheManager::new();
    let cache_mgr_arc = arc::RWArc::new(cache_mgr);

    let shared_v_count = arc::RWArc::new(visitor_count);
    
    let (port, chan) = stream();
    let chan = SharedChan::new(chan);
    let port = SharedPort::new(port);

    // Spawn multiple tasks to response the requests concurrently
    for _ in range(0, CONCURRENT_RESPONSESOR) {
        let child_port = port.clone();
        let do_sched = do_sched.clone();
        let child_cache_arc = cache_mgr_arc.clone();
        
        do spawn {
            let (sm_port, sm_chan) = stream();
            let mut buf = [0, .. FILE_CHUNK_SIZE];
            
            loop {
                child_port.recv(); // wait for new request
                do do_sched.write |sched| {
                    match sched.pqueue.maybe_pop() {
                        None => { /* do nothing */ }
                        Some(msg) => {sm_chan.send(msg);}
                    }
                }
            
                let mut sm: sched_msg = sm_port.recv(); // get new request
                println(fmt!("begin serving file [%?]", sm.file_path));
                                
                if sm.file_path.to_str().ends_with(".shtml") {
                    sm.stream.write("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n".as_bytes());
                    match io::file_reader(sm.file_path) {
                        Err(err) => { println(err); },
                        Ok(reader) => {
                            do reader.each_line() |line| {
                                if line.contains("<!--#exec cmd=\"") {
                                    let start = line.find_str("<!--#exec cmd=\"").unwrap();
                                    let start_cmd = start + 15;
                                    let mut end_cmd = -1;
                                    let mut end = -1;
                                    for i in range(start_cmd+1, line.len()) {
                                        if line.char_at(i) == '"' {
                                            end_cmd = i;
                                        } else if line.char_at(i) == '>' {
                                            end = i + 1;
                                        }
                                        if end_cmd != -1 && end != -1 {
                                            break;
                                        }
                                    }
                                    if end_cmd == -1 || end == -1 || end_cmd >= end {
                                        sm.stream.write(line.as_bytes());
                                    } else {
                                        sm.stream.write(line.slice_to(start).as_bytes());
                                        let cmd = line.slice(start_cmd, end_cmd);
                                        match gash::handle_cmdline(cmd) {
                                            Some(process_output) => {
                                                // Assume we are running on POSIX compliant machine
                                                if process_output.status == 0 {
                                                    sm.stream.write(process_output.output);
                                                } else {
                                                    sm.stream.write(process_output.error);
                                                }
                                            }
                                            None => ()
                                        }
                                        sm.stream.write(line.slice_from(end).as_bytes());
                                    }
                                } else {
                                    sm.stream.write(line.as_bytes());
                                }
                                true
                            };
                        }
                    }
                    
                } else {
                    sm.stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n".as_bytes());
                    if sm.file_size > CACHE_SIZE/2 {
                        // no caching
                        let mut file_reader = file::open(sm.file_path, Open, Read).unwrap();
                        while (!file_reader.eof()) {
                            match file_reader.read(buf) {
                                Some(len) => {sm.stream.write(buf.slice(0, len));}
                                None => {}
                            }
                        }
                    } else { // caching read
                        let last_modified = match file::stat(sm.file_path) {
                            Some(file_stat) => file_stat.modified,
                            None => 0
                        };
                        let path_str = sm.file_path.to_str();
                        let mut need_update = true;
                        
                        do child_cache_arc.write |cache_mgr| {
                            match cache_mgr.modified_map.find(&path_str) {
                                Some(&modified_time) => {
                                    if modified_time == last_modified {
                                        match cache_mgr.cache.get(&path_str) {
                                            Some(data) => {
                                                // get cached file
                                                // TODO: move the network IO operation out of the arc.
                                                // downgrade read?
                                                println("read from cache......................");
                                                sm.stream.write(*data);
                                                need_update = false;
                                            }
                                            None => ()
                                        }
                                    }
                                }
                                None => ()
                            }
                        }
                        
                        if need_update {
                            
                            let mut file_reader = file::open(sm.file_path, Open, Read).unwrap();
                            let mut file_data: ~[u8] = ~[];
                            
                            while (!file_reader.eof()) {
                                match file_reader.read(buf) {
                                    Some(len) => {sm.stream.write(buf.slice(0, len)); file_data.push_all_move(buf.slice(0, len).to_owned());}
                                    None => {}
                                }
                            }
                            
                            let (ptr_port, ptr_chan) = stream();
                            ptr_chan.send(file_data);
                            do child_cache_arc.write |cache_mgr| {
                                let file_data = ptr_port.recv();
                                cache_mgr.cache.put(path_str.clone(), file_data, sm.file_size);
                                cache_mgr.modified_map.swap(path_str.clone(), last_modified);
                            }
                            println("add into cache");
                        }
                    }
                }
                println("finish serving");
            }
        }
    }
    
    let ip = match FromStr::from_str(IP) { Some(ip) => ip, 
                                           None => { println(fmt!("Error: Invalid IP address <%s>", IP));
                                                     return;},
                                         };
    
    let socket = net::tcp::TcpListener::bind(SocketAddr {ip: ip, port: PORT as u16});
    
    println(fmt!("Listening on %s:%d ...", IP, PORT));
    let mut acceptor = socket.listen().unwrap();
    
    for stream in acceptor.incoming() {
        let stream = Cell::new(stream);
        
        // Start a new task to handle the connection
        let child_chan = chan.clone();
        let inc_v_count = shared_v_count.clone();
        let child_add_sched = add_sched.clone();
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
                
                let file_path = ~os::getcwd().push(path.replace("/../", ""));
                if !os::path_exists(file_path) || os::path_is_dir(file_path) {
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
                    // enqueue new request.
                    let file_size = match std::rt::io::file::stat(file_path) {
                                        Some(s) => s.size as uint,
                                        None() => 0,
                    };
                    let msg: sched_msg = sched_msg{priority: 0, stream: stream, file_path: file_path.clone(), file_size: file_size};
                    let (sm_port, sm_chan) = std::comm::stream();
                    sm_chan.send(msg);
                    
                    do child_add_sched.write |sched| {
                        let msg = sm_port.recv();
                        sched.add_sched_msg(msg);
                        //println("new request added to queue");
                    }
                    child_chan.send(""); //notify the new request in queue.
                }
            }
        }
    }
}
