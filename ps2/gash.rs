use std::{io, run, os, path, uint, libc};

use std::libc::funcs::posix88::unistd;
use std::task;


fn handle_cmd(cmd_line: &str, pipe_in: libc::c_int, pipe_out: libc::c_int, pipe_err: libc::c_int) {
    let mut argv: ~[~str] = cmd_line.split_iter(' ').filter(|&x| x != "").transform(|x| x.to_owned()).collect();
    debug!(fmt!("argv %?", argv));
    if argv.len() > 0 {
        //history.push(line);
        let program = argv.remove(0);
        match program {
            ~"help"     => {println("This is a new shell implemented in Rust!")}
            ~"cd"       => {if argv.len()>0 {os::change_dir(&path::PosixPath(argv[0]));}}
            //~"history"  => {for uint::range(0, history.len()) |i| {println(fmt!("%5u %s", i+1, history[i]));}}
            ~"exit"     => {return; }
            //_           => {run::process_status(program, argv);}
            _           => {let mut prog = run::Process::new(program, argv, run::ProcessOptions {
                                                                                        env: None,
                                                                                        dir: None,
                                                                                        in_fd: Some(pipe_in),
                                                                                        out_fd: Some(pipe_out),
                                                                                        err_fd: Some(pipe_err)
                                                                                    });
                             let ret = prog.finish();
                             // close the pipe after the process terminates.
                             // check by ifconfig | tail
                             if pipe_in != 0 {os::close(pipe_in);}
                             if pipe_out != 1 {os::close(pipe_out);}
                             if pipe_err != 2 {os::close(pipe_err);}
                            }
        }//match 
    }//if
}

fn handle_cmdline(cmd_line:&str, bg_flag:bool)
{
    // handle pipes
    // why filter(|&x| x != "") should be removed???
    //let progs: ~[~str] = cmd_line.split_str_iter("|").filter(|&x| x != "").transform(|x| x.to_owned()).collect();
    let progs: ~[~str] = cmd_line.split_str_iter("|").transform(|x| x.to_owned()).collect();
    
    let mut pipes = ~[];
    for uint::range(0, progs.len()-1) |i|{
        pipes.push(os::pipe());
    }
        
    if progs.len() == 1 {
        if bg_flag == false { handle_cmd(progs[0], 0, 1, 2); }
        else {task::spawn_sched(task::SingleThreaded, ||{handle_cmd(progs[0], 0, 1, 2)});}
    } else {
        for uint::range(0, progs.len()) |i| {
            let prog = progs[i].to_owned();
            
            if i == 0 {
                let pipe_i = pipes[i];
                task::spawn_sched(task::SingleThreaded, ||{handle_cmd(prog, 0, pipe_i.out, 2)});
            } else if i == progs.len() - 1 {
                let pipe_i_1 = pipes[i-1];
                if bg_flag == true {
                    task::spawn_sched(task::SingleThreaded, ||{handle_cmd(prog, pipe_i_1.in, 1, 2)});
                } else {
                    handle_cmd(prog, pipe_i_1.in, 1, 2);
                }
            } else {
                let pipe_i = pipes[i];
                let pipe_i_1 = pipes[i-1];
                task::spawn_sched(task::SingleThreaded, ||{handle_cmd(prog, pipe_i_1.in, pipe_i.out, 2)});
            }
        }
    }
}

fn main() {
    static CMD_PROMPT: &'static str = "vash > ";
    //let mut history: ~[~str] = ~[];
    
    loop {
        print(CMD_PROMPT);
        // consider how to handle \ | < > &, and shortcuts such as Ctrl+C.
        let mut cmd_line = io::stdin().read_line();
        
        // check &, background?
        let mut bg_flag = false;
        let amp_pos = cmd_line.find('&');
        if amp_pos != None {
            cmd_line = cmd_line.slice_to(amp_pos.get()).to_owned();
            bg_flag = true;
        }
        
        handle_cmdline(cmd_line, bg_flag);
    }
}
