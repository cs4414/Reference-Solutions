//
// zhttpto.rs
//
// Reference solution for PS1
// Running on Rust 0.8
//
// Note that this code has serious security risks!  You should not run it 
// on any system with access to sensitive files.
//
// Special thanks to Kiet Tran for providing code we incorporated into this.
// 
// University of Virginia - cs4414 Fall 2013
// Weilin Xu and David Evans
// Version 0.2

extern mod extra;

use std::rt::io::*;
use std::rt::io::net::ip::{SocketAddr};
use std::io::println;
use std::cell::Cell;
use std::task;
use std::{os, str, path};
use std::vec;
use extra::getopts::*;

static PORT:    int = 4414;
static IP: &'static str = "127.0.0.1";
static mut visitor_count: uint = 0;
static FILE_CHUNK_BUF_SIZE: uint = 512000;  // default size of buffer (bytes)

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
    let www_dir = match matches.opt_str("dir") {Some(dir) => {dir}, None => {"./".to_owned()}};
    
    println("\nStarting zhttpto...");
    printfln!("Serving at %s", path::PosixPath(www_dir).to_str());
    
    /* Finish processing program arguments and initiate the parameters. */
    
    os::change_dir(&path::PosixPath(www_dir));
    
    let ip = match FromStr::from_str(ip_str) { Some(ip) => ip, 
                                           None => { println(fmt!("Error: Invalid IP address <%s>", IP));
                                                     return;},
                                         };
                                         
    let socket = net::tcp::TcpListener::bind(SocketAddr {ip: ip, port: port_int as u16});
    println(fmt!("Listening on %s:%d ...", ip_str, port_int));
    let mut acceptor = socket.listen().unwrap();
    
    // we can limit the incoming connection count.
    //for stream in acceptor.incoming().take(10 as uint) {
    for stream in acceptor.incoming() {
        println!("Saw connection!");
        let stream = Cell::new(stream);
        // Start a task to handle the connection
        do task::spawn {
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
                
                let file_path = &os::getcwd().push(path.replace("/../", ""));
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
                    println("page replied.");
                }
                else {
                    /*
                    println(fmt!("serve file: %?", file_path));
                    match io::read_whole_file(file_path) {
                        Ok(file_data) => {
                            stream.write(file_data);
                            println("file replied.");
                        }
                        Err(err) => {
                            println(err);
                        }
                    }
                    */
                    let mut file_chunk_buf: ~[u8] = vec::with_capacity(FILE_CHUNK_BUF_SIZE);
                    unsafe {vec::raw::set_len(&mut file_chunk_buf, FILE_CHUNK_BUF_SIZE);} // File_reader.read() doesn't recognize capacity, but len() instead. A wrong design?
                    
                    printfln!("%s", file_path.components[file_path.components.len()-1]);
                    let mut file_reader = file::open(file_path, Open, Read).unwrap();
                    stream.write("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream; charset=UTF-8\r\n\r\n".as_bytes());
                    while (!file_reader.eof()) {
                        match file_reader.read(file_chunk_buf) {
                            Some(len) => {stream.write(file_chunk_buf.slice(0, len));}
                            None => {}
                        }
                    }
                }
            }
            println!("connection terminates")
        }
    }
}
