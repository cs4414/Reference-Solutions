//
// gash.rs
//
// Starting code for PS2
// Running on Rust 0.9
//
// University of Virginia - cs4414 Fall 2013
// Weilin Xu, David Evans
// Version 0.4
//

use std::{io, run};
use std::io::buffered::BufferedReader;
use std::io::stdin;

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
                    
                    _           =>  {
                                        if self.cmd_exists(program) {
                                            run::process_status(program, argv);
                                        } else {
                                            println!("{:s}: command not found", program);
                                        }
                                    }
                    
                }
            }
        }
    }
    
    fn cmd_exists(&mut self, cmd_path: &str)  -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }
}

fn main() {
    let mut gash = Shell::new("gash > ");
    gash.run();
}
