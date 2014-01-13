//
// gash.rs
//
// Starting code for PS2
// Running on Rust 0.9
//
// University of Virginia - cs4414 Spring 2014
// Weilin Xu, David Evans
// Version 0.4
//

extern mod extra;

use std::{io, run, os, libc};
use std::io::buffered::BufferedReader;
use std::io::stdin;
use extra::getopts;

struct Shell {
    history: ~[~str],
    cmd_prompt: ~str,
}

impl Shell {
    fn new(prompt_str: &str) -> Shell {
        Shell {
            history: ~[],
            cmd_prompt: prompt_str.to_owned(),
        }
    }
    
    fn run(&mut self) {
        let mut stdin = BufferedReader::new(stdin());
        
        loop {
            print(self.cmd_prompt);
            io::stdio::flush();
            
            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim().to_owned();
            
            if cmd_line.len() > 0 {
                self.history.push(cmd_line.to_owned());
            }
            
            let mut argv: ~[~str] =
                cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
        
            if argv.len() > 0 {
                let program = argv.remove(0);
                match program {
                    ~"exit"     =>  { return; }
                    
                    ~"history"  =>  {
                                        for i in range(0, self.history.len()) {
                                            println(format!("{:u} {:s}", i+1, self.history[i]));
                                        }
                                    }
                    
                    _           =>  { self.run_cmd(program, argv); }
                    
                }
            }
        }
    }
    
    fn run_cmdline(&mut self, cmd_line: &str) {
        let mut argv: ~[~str] =
            cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
        if argv.len() > 0 {
            let program: ~str = argv.remove(0);
            self.run_cmd(program, argv);
        }
    }
    
    fn run_cmd(&mut self, program: &str, argv: &[~str]) {
        if self.cmd_exists(program) {
            run::process_status(program, argv);
        } else {
            println!("{:s}: command not found", program);
        }
    }
    
    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }
}

fn get_cmdline_from_args() -> Option<~str> {
    fn print_usage(program: &str) {
        println!("Usage: {:s} [options]", program);
        println("-c            \tcommand");
        println("-h --help     \tUsage");
    }
    
    /* Begin processing program arguments and initiate the parameters. */
    let args = os::args();
    let program = args[0].clone();
    
    let opts = ~[
        getopts::optopt("c"),
        getopts::optflag("h"),
        getopts::optflag("help")
    ];
    let matches = match getopts::getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_err_msg()) }
    };
    if matches.opt_present("h") || matches.opt_present("help") {
        print_usage(program);
        unsafe { libc::exit(1); }
    }
    
    if matches.opt_present("c") {
        let cmd_str = match matches.opt_str("c") {
                                                Some(cmd_str) => {cmd_str.to_owned()}, 
                                                None => {~""}
                                              };
        return Some(cmd_str);
    } else {
        return None;
    }
}

fn main() {
    let opt_cmd_line = get_cmdline_from_args();
    
    match opt_cmd_line {
        Some(cmd_line) => Shell::new("").run_cmdline(cmd_line),
        None           => Shell::new("gash > ").run()
    }
}
