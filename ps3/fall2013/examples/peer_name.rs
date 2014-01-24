// Here's an example code that can get the peer IP address of any incoming connection from arbitrary locations
// Running in Rust 0.8
// Author: Weilin Xu <xuweilin@virginia.edu>

use std::rt::io::*;
use std::rt::io::net::ip::SocketAddr;
use std::io::println;
use std::from_str::FromStr;

static IP: &'static str = "0.0.0.0";
static PORT:    int = 4414;

fn main() {
    let ip = match FromStr::from_str(IP) { Some(ip) => ip, 
                                           None => { println(fmt!("Error: Invalid IP address <%s>", IP));
                                                     return;},
                                         };
    
    let socket = net::tcp::TcpListener::bind(SocketAddr {ip: ip, port: PORT as u16});
    println(fmt!("Listening on %s:%d ...", ip.to_str(), PORT));
    let mut acceptor = socket.listen();
    
    loop {
        match acceptor.accept() {
            Some(s) => { let mut stream = s;
                         match stream.peer_name() {
                            Some(pn) => {println(fmt!("Peer address: %s", pn.to_str()));},
                            None => ()
                         }
                       },
            None => ()
        }
    }
}
