//
// gash.rs
//
// Starting code for PS2
// Running on Rust 0.9
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu, David Evans
// Version 0.3
//

// TODO: add support to gash arguments to make non-interactive tests easier.
use std::{io, run};
use std::io::buffered::BufferedReader;
use std::io::stdin;

fn main() {
    static CMD_PROMPT: &'static str = "gash > ";
    
    let mut stdin = BufferedReader::new(stdin());
    
    loop {
        print(CMD_PROMPT);
        io::stdio::flush();
        
        let line = stdin.read_line().unwrap();
        let cmd_line = line.trim().to_owned();
        
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
        
        if argv.len() > 0 {
            let program = argv.remove(0);
            match program {
                ~"exit"     => {return; }
                _           => {run::process_status(program, argv);}
            }
        }
    }
}
