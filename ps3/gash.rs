//
// gash module
// Running on Rust 0.8

use std::{run, os, libc};
use std::task;

fn get_fd(fpath: &str, mode: &str) -> libc::c_int {
    #[fixed_stack_segment]; #[inline(never)];

    unsafe {
        let fpathbuf = fpath.to_c_str().unwrap();
        let modebuf = mode.to_c_str().unwrap();
        return libc::fileno(libc::fopen(fpathbuf, modebuf));
    }
}

fn exit(status: libc::c_int) {
    #[fixed_stack_segment]; #[inline(never)];
    unsafe { libc::exit(status); }
}

fn _handle_cmd(cmd_line: &str, pipe_in: libc::c_int,
               pipe_out: libc::c_int, pipe_err: libc::c_int, output: bool) -> Option<run::ProcessOutput> {
    let out_fd = pipe_out;
    let in_fd = pipe_in;
    let err_fd = pipe_err;
    
    let mut argv: ~[~str] =
        cmd_line.split_iter(' ').filter_map(|x| if x != "" { Some(x.to_owned()) } else { None }).to_owned_vec();
    
    if argv.len() > 0 {
        let program = argv.remove(0);
        let (out_opt, err_opt) = if output { (None, None) } else { (Some(out_fd), Some(err_fd))};
        let mut prog = run::Process::new(program, argv, run::ProcessOptions {
                                                                env: None,
                                                                dir: None,
                                                                in_fd: Some(in_fd),
                                                                out_fd: out_opt,
                                                                err_fd: err_opt
                                                            });
        let output_opt = if output { Some(prog.finish_with_output()) } 
                         else { prog.finish(); None };

        // close the pipes after process terminates.
        if in_fd != 0 {os::close(in_fd);}
        if out_fd != 1 {os::close(out_fd);}
        if err_fd != 2 {os::close(err_fd);}

        return output_opt;
    }
    return None;
}

fn handle_cmd(cmd_line: &str, pipe_in: libc::c_int, pipe_out: libc::c_int, pipe_err: libc::c_int) {
    _handle_cmd(cmd_line, pipe_in, pipe_out, pipe_err, false);
}

fn handle_cmd_with_output(cmd_line: &str, pipe_in: libc::c_int) -> Option<run::ProcessOutput> {
    return _handle_cmd(cmd_line, pipe_in, -1, -1, true);
}

pub fn handle_cmdline(cmd_line: &str) -> Option<run::ProcessOutput> {
    // handle pipes
    let progs: ~[~str] = cmd_line.split_iter('|').map(|x| x.trim().to_owned()).collect();
    
    let mut pipes = ~[];
    for _ in range(0, progs.len()-1) {
        pipes.push(os::pipe());
    }
        
    if progs.len() == 1 {
        return handle_cmd_with_output(progs[0], 0);
    } else {
        let mut output_opt = None;
        for i in range(0, progs.len()) {
            let prog = progs[i].to_owned();
            
            if i == 0 {
                let pipe_i = pipes[i];
                task::spawn_sched(task::SingleThreaded, ||{handle_cmd(prog, 0, pipe_i.out, 2)});
            } else if i == progs.len() - 1 {
                let pipe_i_1 = pipes[i-1];
                output_opt = handle_cmd_with_output(prog, pipe_i_1.input);
            } else {
                let pipe_i = pipes[i];
                let pipe_i_1 = pipes[i-1];
                task::spawn_sched(task::SingleThreaded, ||{handle_cmd(prog, pipe_i_1.input, pipe_i.out, 2)});
            }
        }
        return output_opt;
    }
}
