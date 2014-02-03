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

use std::{io, run, os, libc, path};
use std::io::buffered::BufferedReader;
use std::io::stdin;
use extra::getopts;
use std::io::signal::{Listener, Interrupt};
use std::libc::funcs::posix88::signal;

struct Shell {
    history: ~[~str],
    cmd_prompt: ~str,
    fg_pid: libc::pid_t
}

impl Shell {
    fn new(prompt_str: &str) -> Shell {
        Shell {
            history: ~[],
            cmd_prompt: prompt_str.to_owned(),
            fg_pid: -1,
        }
    }
    
    // run shell
    fn run(&mut self) {
        self.register_signal_handler();
        let mut stdin = BufferedReader::new(stdin());
        
        loop {
            print(self.cmd_prompt);
            io::stdio::flush();
            
            let line = stdin.read_line().unwrap();
            let cmd_line = line.trim().to_owned();
            
            if cmd_line.len() > 0 {
                self.history.push(cmd_line.to_owned());
            }
            
            let program = cmd_line.splitn(' ', 1).nth(0).expect("no program");
        
            match program {
                // no need to parse the arguements for several internal commands.   
                ""          =>  { continue; }
                "exit"      =>  { self.exit(0); }
                "help"      =>  { println("This is a new shell implemented in Rust!") }
                "history"   =>  {
                                    for i in range(0, self.history.len()) {
                                        println(format!("{:u} {:s}", i+1, self.history[i]));
                                    }
                                }
                _           =>  { self.run_cmdline(cmd_line); }
                
            }
        }
    }
    
    // run command line, called by self.run() or external -c parameter.
    // You will need to update this function to support more complex shell features.
    fn run_cmdline(&mut self, cmd_line: &str) {
        // handle background sign &
        let mut bg_flag = false;
        let mut cmd_line: ~str = cmd_line.to_owned();
        if cmd_line.ends_with("&") {
            cmd_line = cmd_line.trim_right_chars(&'&').to_owned();
            bg_flag = true;
        }
        
        // handle pipelines
        let progs: ~[~str] =
            cmd_line.split('|').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
        
        let mut pipes: ~[os::Pipe] = ~[];
        
        // create pipes
        if (progs.len() > 1) {
            for _ in range(0, progs.len()-1) {
                pipes.push(os::pipe());
            }
        }
        
        if progs.len() == 1 {
            self.run_single_cmd(progs[0], 0, 1, 2, bg_flag);
        } else {
            for i in range(0, progs.len()) {
                let prog = progs[i].to_owned();
                
                if i == 0 { // the first one
                    self.run_single_cmd(prog, 0, pipes[i].out, 2, true);
                } else if i == progs.len() - 1 { // the last one
                    self.run_single_cmd(prog, pipes[i-1].input, 1, 2, bg_flag);
                } else { // the middle ones
                    self.run_single_cmd(prog, pipes[i-1].input, pipes[i].out, 2, true);
                }
            }
        }
    }
    
    // run a single command line, probably with redirection sign >, definitly without pipelines | and background sign &.
    fn run_single_cmd(&mut self, cmd_line: &str, pipe_in: libc::c_int, pipe_out: libc::c_int, pipe_err: libc::c_int, bg: bool) {
        let mut argv = self.parse_argv(cmd_line);
    
        if argv.len() <= 0 {
            // empty command line
            return;
        }
        
        let mut out_fd = pipe_out;
        let mut in_fd = pipe_in;
        let err_fd = pipe_err;
        
        
        let mut i = 0;
        // found problem on redirection
        // `ping google.com | grep 1 > ping.txt &` didn't work
        // because grep won't flush the buffer until terminated (only) by SIGINT.
        while (i < argv.len()) {
            if (argv[i] == ~">") {
                argv.remove(i);
                out_fd = self.get_fd(argv.remove(i), "w");
            } else if (argv[i] == ~"<") {
                argv.remove(i);
                in_fd = self.get_fd(argv.remove(i), "r");
            }
            i += 1;
        }
        
        let out_fd = out_fd;
        let in_fd = in_fd;
        
        if argv.len() <= 0 {
            // invalid command line
            return;
        }
        
        let program = argv.remove(0);
        match program {
            ~"cd"       => { if argv.len()>0 { os::change_dir(&path::Path::new(argv[0])); } }
            _           => {    if !self.cmd_exists(program) {
                                    println!("{:s}: command not found", program);
                                    return;
                                } else {
                                    //println!("Program: {:s}, in_fd: {:d}, out_fd: {:d}, err_fd: {:d}", program, in_fd, out_fd, err_fd);
                                    let opt_prog = run::Process::new(program, argv, run::ProcessOptions {
                                                                                                env: None,
                                                                                                dir: None,
                                                                                                in_fd: Some(in_fd),
                                                                                                out_fd: Some(out_fd),
                                                                                                err_fd: Some(err_fd)
                                                                                            });
                                    
                                    let mut prog = opt_prog.expect("Error: creating process error.");
                                    if in_fd != 0 {os::close(in_fd);}
                                    if out_fd != 1 {os::close(out_fd);}
                                    if err_fd != 2 {os::close(err_fd);}

                                     if !bg {
                                        self.fg_pid = prog.get_id();
                                        prog.finish();
                                        io::stdio::flush();
                                        //println(program + " terminated.");
                                        
                                     } else {
                                        let (p_port, p_chan) = Chan::new();
                                        p_chan.send(prog);
                                        do spawn {
                                            let mut prog: run::Process = p_port.recv();
                                            
                                            prog.finish(); 
                                            io::stdio::flush();
                                            //println(program + " terminated.");
                                        }
                                     }
                                 }
                            }
        }//match program
    
    }
    
    // input: a single command line
    // output: a vector of arguments. The program name is put in the first place.
    // notes: arguments can be separated by space(s), ""  
    fn parse_argv(&mut self, cmd_line: &str) -> ~[~str] {
        //let argv: ~[~str] = 
        //        cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
        
        let mut argv: ~[~str] = ~[];
        let group: ~[~str] =
            cmd_line.split('\"').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
        for i in range(0, group.len()) {
            if i % 2 == 0 {
                // split by " "
                argv.push_all_move(group[i].split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec());
            } else {
                argv.push(group[i].clone());
            }
        
        }
        
        return argv;
    }
    
    fn cmd_exists(&mut self, cmd_path: &str) -> bool {
        let ret = run::process_output("which", [cmd_path.to_owned()]);
        return ret.expect("exit code error.").status.success();
    }
    
    fn get_fd(&mut self, fpath: &str, mode: &str) -> libc::c_int {
        unsafe {
            let fpathbuf = fpath.to_c_str().unwrap();
            let modebuf = mode.to_c_str().unwrap();
            return libc::fileno(libc::fopen(fpathbuf, modebuf));
        }
    }
    
    fn register_signal_handler(&mut self) {
        let fgpid = self.fg_pid;

        spawn(proc() {
            //TODO: Unregister the listener once exit.
            let mut listener = Listener::new();
            let ret = listener.register(Interrupt);
            
            if ret {
                loop {
                    match listener.port.recv() {
                        Interrupt => { if fgpid != -1 { unsafe { signal::kill(fgpid, libc::SIGINT); }}},
                        _ => (),
                    }
                }
            } else {
                println("Warning: registering signal handler fails.");
            }
            }
        );
    }
    
    fn exit(&mut self, status: int) {
        unsafe { libc::exit(status as i32); } 
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
        None           => Shell::new("gash> ").run()
    }
}
