//
// gash.rs
//
// Reference solution for PS2
// Running on Rust 0.9
//
// University of Virginia - cs4414 Fall 2013
// Weilin Xu, David Evans
// Version 0.3
//

use std::{io, run, os, path, libc};
use std::io::buffered::BufferedReader;
use std::io::stdin;
use std::io::signal::{Listener, Interrupt};
use std::libc::funcs::posix88::signal;

static mut fg_pid: libc::pid_t = 1;

fn cmd_exists(cmd_path: &str)  -> bool {
    // System commands or executables in a specific path.
    let ret = run::process_output("which", [cmd_path.to_owned()]);
    return ret.expect("exit code error.").status.success();
}

fn register_signal_handler() {
    do spawn {
        let mut listener = Listener::new();
        let ret = listener.register(Interrupt);
        
        if ret == true {
            loop {
                match listener.port.recv() {
                    Interrupt => unsafe { signal::kill(fg_pid, libc::SIGINT); },
                    _ => (),
                }
            }
        } else {
            println("Warning: registering signal handler fails.");
        }
    }
}

fn get_fd(fpath: &str, mode: &str) -> libc::c_int {
    unsafe {
        let fpathbuf = fpath.to_c_str().unwrap();
        let modebuf = mode.to_c_str().unwrap();
        return libc::fileno(libc::fopen(fpathbuf, modebuf));
    }
}

fn exit(status: libc::c_int) {
    unsafe { libc::exit(status); }
}

fn handle_cmd(cmd_line: &str, pipe_in: libc::c_int, pipe_out: libc::c_int, pipe_err: libc::c_int, bg: bool) {
    let mut out_fd = pipe_out;
    let mut in_fd = pipe_in;
    let err_fd = pipe_err;
    
    let mut argv: ~[~str] =
        cmd_line.split(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    let mut i = 0;
    // found problem on redirection
    // `ping google.com | grep 1 > ping.txt &` didn't work
    // because grep won't flush the buffer until terminated (only) by SIGINT.
    while (i < argv.len()) {
        if (argv[i] == ~">") {
            argv.remove(i);
            out_fd = get_fd(argv.remove(i), "w");
        } else if (argv[i] == ~"<") {
            argv.remove(i);
            in_fd = get_fd(argv.remove(i), "r");
        }
        i += 1;
    }
    
    let out_fd = out_fd;
    let in_fd = in_fd;
    
    if argv.len() > 0 {
        let program = argv.remove(0);
        match program {
            ~"help"     => {println("This is a new shell implemented in Rust!")}
            ~"cd"       => {if argv.len()>0 {os::change_dir(&path::Path::new(argv[0]));}}
            //global variable?
            //~"history"  => {for i in range(0, history.len()) {println(fmt!("%5u %s", i+1, history[i]));}}
            ~"exit"     => {exit(0);}
            _           => {if !cmd_exists(program) {
                                println!("{:s}: command not found", program);
                            } else {
                                let opt_prog = run::Process::new(program, argv, run::ProcessOptions {
                                                                                            env: None,
                                                                                            dir: None,
                                                                                            in_fd: Some(in_fd),
                                                                                            out_fd: Some(out_fd),
                                                                                            err_fd: Some(err_fd)
                                                                                        });
                                 //
                                 let mut prog = opt_prog.expect("Error: creating process error.");
                                 if !bg {
                                    unsafe{fg_pid = prog.get_id();}
                                    prog.finish(); 
                                    // close the pipes after process terminates.
                                    println(program + " terminated.");
                                    if in_fd != 0 {os::close(in_fd); println(program + " close in_fd");}
                                    if out_fd != 1 {os::close(out_fd); println(program + " close out_fd");}
                                    if err_fd != 2 {os::close(err_fd); println(program + " close err_fd");}
                                 } else {
                                    let (p_port, p_chan) = Chan::new();
                                    p_chan.send(prog);
                                    do spawn {
                                        let mut prog: run::Process = p_port.recv();
                                        
                                        prog.finish(); 
                                        // close the pipes after process terminates.
                                        println(program + " terminated.");
                                        if in_fd != 0 {os::close(in_fd); println(program + " close in_fd");}
                                        if out_fd != 1 {os::close(out_fd); println(program + " close out_fd");}
                                        if err_fd != 2 {os::close(err_fd); println(program + " close err_fd");}
                                    }
                                 }
                             }
                            }
        }//match program
    }//if
}

fn handle_cmdline(cmd_line:&str, bg_flag:bool)
{
    // handle pipes
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
        handle_cmd(progs[0], 0, 1, 2, bg_flag);
    } else {
        for i in range(0, progs.len()) {
            let prog = progs[i].to_owned();
            
            if i == 0 {
                let pipe_i = pipes[i];
                handle_cmd(prog, 0, pipe_i.out, 2, true);
            } else if i == progs.len() - 1 {
                let pipe_i_1 = pipes[i-1];
                handle_cmd(prog, pipe_i_1.input, 1, 2, bg_flag);
            } else {
                let pipe_i = pipes[i];
                let pipe_i_1 = pipes[i-1];
                handle_cmd(prog, pipe_i_1.input, pipe_i.out, 2, true);
            }
        }
    }
}

fn main() {
    register_signal_handler();
    static CMD_PROMPT: &'static str = "gash > ";
    let mut history: ~[~str] = ~[];
    
    let mut stdin = BufferedReader::new(stdin());
    
    loop {
        print(CMD_PROMPT);
        io::stdio::flush();
        
        let line = stdin.read_line().unwrap();
        
        let mut cmd_line = line.trim().to_owned();
        
        if cmd_line.len() > 0 {
            history.push(cmd_line.to_owned());
        }
        let mut bg_flag = false;
        if cmd_line.ends_with("&") {
            cmd_line = cmd_line.trim_right_chars(&'&').to_owned();
            bg_flag = true;
        }
        
        if cmd_line == ~"exit" {
            exit(0);
        } else if cmd_line == ~"history" {
            for i in range(0, history.len()) {
                println(format!("{:u} {:s}", i+1, history[i]));
            }
        } else {
            handle_cmdline(cmd_line, bg_flag);
        }
    }
}
