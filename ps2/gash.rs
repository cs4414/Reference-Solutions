use std::{io, run, os, path, uint, libc};

//use std::libc::funcs::posix88::unistd;
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
           // ~"history"  => {for uint::range(0, history.len()) |i| {println(fmt!("%5u %s", i+1, history[i]));}}
            _           => {let mut prog = run::Process::new(program, argv, run::ProcessOptions {
                                                                                        env: None,
                                                                                        dir: None,
                                                                                        in_fd: Some(pipe_in),
                                                                                        out_fd: Some(pipe_out),
                                                                                        err_fd: Some(pipe_err)
                                                                                    });
                             prog.finish();
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
    let count_progs: uint = progs.len()-1;

    let mut pipes = ~[];
    for uint::range(0u, count_progs) |_|{
        pipes.push(os::pipe());
    }
    
    // Find whether input or output is being redirected, and replace stdin/stdout if necessary
    let mut last_prog = copy(progs[count_progs]);
    let stdin = if progs[count_progs].find('<') != None {
        let mut exploded = progs[count_progs].split_str_iter("<");
        let filename = exploded.nth(1).get().trim();
        last_prog = copy(exploded.nth(0).get().trim().to_owned());
        unsafe{filename.as_c_str(|f| "r".as_c_str(|mode| libc::fopen(f, mode)))}
    } else {
        0 as *std::libc::types::common::c95::FILE
    };

    let stdout = if progs[count_progs].find('>') != None {
        let mut exploded = progs[count_progs].split_str_iter(">");
        let filename = exploded.nth(1).get().trim();
        last_prog = copy(exploded.nth(0).get().trim().to_owned());
        unsafe{filename.as_c_str(|f| "w".as_c_str(|mode| libc::fopen(f, mode)))}
    } else {
        1 as *std::libc::types::common::c95::FILE
    };

    let prog_final = copy(last_prog);
    if progs.len() == 1 {
        if bg_flag == false { handle_cmd(progs[0], 0, 1, 2); }
        else {task::spawn_sched(task::SingleThreaded, ||{handle_cmd(progs[0], stdin as i32, stdout as i32, 2)});}
    } else {
        for uint::range(0, progs.len()) |i| {
            let prog = progs[i].to_owned();
            
            if i == 0 {
                let pipe_i = pipes[i];
                task::spawn_sched(task::SingleThreaded, ||{handle_cmd(prog, stdin as i32, pipe_i.out, 2)});
            } else if i == progs.len() - 1 {
                let pipe_i_1 = pipes[i-1];
                if bg_flag == true {
                    task::spawn_sched(task::SingleThreaded, ||{handle_cmd(prog.split_str_iter(">").nth(0).get().trim().to_owned(), pipe_i_1.in, stdout as i32, 2)});
                } else {
                    handle_cmd(prog_final, pipe_i_1.in, 1, 2);
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
    static CMD_PROMPT: &'static str = "gash > ";
    let mut history: ~[~str] = ~[];
    
    loop {
        print(CMD_PROMPT);
        // consider how to handle \ | < > &, and shortcuts such as Ctrl+C.
        let mut cmd_line = io::stdin().read_line();
        history.push(copy(cmd_line));
        // check &, background?
        let mut bg_flag = false;
        let amp_pos = cmd_line.find('&');
        if amp_pos != None {
            cmd_line = cmd_line.slice_to(amp_pos.get()).to_owned();
            bg_flag = true;
        }
        if cmd_line == ~"exit" {
            break;
        } else if cmd_line == ~"history" {
            for uint::range(0, history.len()) |i| {
                println(fmt!("%5u %s", i+1, history[i]));
            }
        } else {
            handle_cmdline(cmd_line, bg_flag);
        }
    }
}
