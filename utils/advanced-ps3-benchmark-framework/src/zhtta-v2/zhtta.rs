//
// zhtta.rs
//
// Running on Rust 0.8
//
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

static PORT:    int = 4414;
static IP: &'static str = "127.0.0.1";
static mut visitor_count: uint = 0;
static FILE_CHUNK_BUF_SIZE: int = 512000; //bytes

struct sched_msg {
    stream: Option<std::rt::io::net::tcp::TcpStream>,
    filepath: ~std::path::PosixPath
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
    let www_dir = match matches.opt_str("dir") {Some(dir) => {dir}, None => {"./".to_owned()}};
    
    println("\nStarting zhtta...");
    printfln!("Size of file chunk buffer: %d bytes", bufsize);
    printfln!("Serving at %s", path::PosixPath(www_dir).to_str());
    
    /* Finish processing program arguments and initiate the parameters. */
    
    os::change_dir(&path::PosixPath(www_dir));

    let req_vec: ~[sched_msg] = ~[];
    let shared_req_vec = arc::RWArc::new(req_vec);
    let add_vec = shared_req_vec.clone();
    let take_vec = shared_req_vec.clone();
    
    let (port, chan) = stream();
    let chan = SharedChan::new(chan);
    
    // dequeue file requests, and send responses.
    // FIFO
    do spawn {
        let (sm_port, sm_chan) = stream();
        let mut file_chunk_buf: ~[u8] = vec::with_capacity(bufsize as uint);
        unsafe {vec::raw::set_len(&mut file_chunk_buf, bufsize as uint);} // File_reader.read() doesn't recognize capacity, but len() instead. A wrong design?
        
        loop {
            port.recv(); // wait for arrving notification
            do take_vec.write |vec| {
                if ((*vec).len() > 0) {
                    //println(fmt!("queue size before popping: %u", (*vec).len()));
                    let tf_opt: Option<sched_msg> = (*vec).shift_opt();
                    //println(fmt!("queue size after popping: %u", (*vec).len()));
                    let tf = tf_opt.unwrap();

                    //println(fmt!("shift from queue, size: %ud", (*vec).len()));
                    sm_chan.send(tf); // send the request to send-response-task to serve.
                }
            }
            let mut tf: sched_msg = sm_port.recv(); // wait for the dequeued request to handle
            // Print the serving file's name.
            printfln!("%s", tf.filepath.components[tf.filepath.components.len()-1]);
            let mut file_reader = file::open(tf.filepath, Open, Read).unwrap();
            tf.stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n".as_bytes());
            while (!file_reader.eof()) {
                match file_reader.read(file_chunk_buf) {
                    Some(len) => {tf.stream.write(file_chunk_buf.slice(0, len));}
                    None => {}
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
        let child_add_vec = add_vec.clone();
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
                    let msg: sched_msg = sched_msg{stream: stream, filepath: file_path.clone()};
                    let (sm_port, sm_chan) = std::comm::stream();
                    sm_chan.send(msg);
                    
                    do child_add_vec.write |vec| {
                        let msg = sm_port.recv();
                        //println(fmt!("add to queue: %?", msg.filepath.filename().unwrap()));
                        //println(fmt!("queue size before pushing: %u", (*vec).len()));
                        (*vec).push(msg); // enqueue new request.
                        //println(fmt!("queue size after pushing: %u", (*vec).len()));
                        
                    }
                    child_chan.send(""); //notify the new arriving request.
                    //println(fmt!("get file request: %?", file_path));
                }
            }
            //println!("connection terminates")
        }
    }
}
