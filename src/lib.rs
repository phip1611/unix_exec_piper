/*
    MIT License

    Copyright (c) 2020 Philipp Schuster

    Permission is hereby granted, free of charge, to any person obtaining a copy
    of this software and associated documentation files (the "Software"), to deal
    in the Software without restriction, including without limitation the rights
    to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
    copies of the Software, and to permit persons to whom the Software is
    furnished to do so, subject to the following conditions:

    The above copyright notice and this permission notice shall be included in all
    copies or substantial portions of the Software.

    THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
    IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
    FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
    AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
    LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
    OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
    SOFTWARE.
*/

use crate::data::{CmdChain, BasicCmd};
use crate::pipe::Pipe;

mod libc_util;
pub mod data;
mod pipe;

/// Runs a command chain. The parent process creates n childs and
/// connects them (stdout => stdin) together via pipes.
pub fn execute_piped_cmd_chain(cmds: &CmdChain) -> Vec<libc::pid_t> {
    let mut pids: Vec<libc::pid_t> = vec![];

    let mut pipe_to_current = Option::None;
    let mut pipe_to_next = Option::None;
    for i in 0..cmds.length() {
        let cmd = &cmds.cmds()[i];

        if pipe_to_next.is_some() {
            pipe_to_current.replace(
                pipe_to_next.take().unwrap()
            );
        }

        if !cmd.is_last() {
            pipe_to_next.replace(Pipe::new());
        }

        let pid = unsafe { libc::fork() };
        if pid == -1 {
            panic!("Fork failed! {}", errno::errno());
        }

        // parent code
        if pid > 0 {
            pids.push(pid);

            // We MUST close all FDs in the Parent
            if pipe_to_current.is_some() {
                pipe_to_current.as_mut().unwrap().parent_close_all();
            }
        }
        // child code
        else {
            // handle optional initial '< in.file' redirect
            if cmd.is_first() && cmd.in_red_path().is_some() {
                initial_ir(cmd);
            }
            // handle optional final '> out.file' redirect
            if cmd.is_last() && cmd.out_red_path().is_some() {
                final_or(cmd);
            }

            if pipe_to_current.is_some() {
                pipe_to_current.as_mut().unwrap().as_read_end();
            }
            if pipe_to_next.is_some() {
                pipe_to_next.as_mut().unwrap().as_write_end();
            }

            let _res = unsafe {
                libc::execvp(
                    cmd.executable_cstring().as_ptr(),
                    cmd.args_to_c_argv()
                )
            };
            panic!("Exec failed! {}", errno::errno());
        }
    }

    // sync wait for all
    if !cmds.background() {
        for pid in &pids {
            let mut status: libc::c_int = 0;
            let res = unsafe { libc::waitpid(*pid, &mut status as * mut libc::c_int, 0) };
            if res == -1 {
                panic!("Failure during waitpid! {}", errno::errno());
            } else {
                println!("Process {} finished with status code {}", pid, status);
            }
        }
    }

    // else it's up to the caller
    // probably using waitpid() with flag W_NOHANG

    pids
}

/// Handles initial input redirect (from file).
pub fn initial_ir(cmd: &BasicCmd) {
    let fd = unsafe {
        libc::open(
            cmd.in_red_path_cstring().unwrap().as_ptr(),
            libc::O_RDONLY,
            0644,
        )
    };
    if fd == -1 {
        panic!("Input redirect path {} doesn't exist! {}", cmd.in_red_path().as_ref().unwrap(), errno::errno());
    }
    let ret = unsafe { libc::dup2(fd, libc::STDIN_FILENO) };
    if ret == -1 {
        panic!("Error dup2() input redirect! {}", errno::errno());
    }
}

/// Handles final output redirect (to file).
pub fn final_or(cmd: &BasicCmd) {
    let fd = unsafe {
        libc::open(
            cmd.out_red_path_cstring().unwrap().as_ptr(),
            libc::O_WRONLY | libc::O_CREAT
        )
    };
    if fd == -1 {
        panic!("Output redirect path {} doesn't exist! {}", cmd.out_red_path().as_ref().unwrap(), errno::errno());
    }
    let ret = unsafe { libc::dup2(fd, libc::STDOUT_FILENO) };
    if ret == -1 {
        panic!("Error dup2() output redirect! {}", errno::errno());
    }
}

#[cfg(test)]
mod tests {
    use crate::data::{CmdChainBuilder, BasicCmdBuilder, Builder};
    use crate::execute_piped_cmd_chain;

    #[test]
    fn test_execute_chain() {
        // this test works if "2" is printed to stdout

        let cmd_chain = CmdChainBuilder::new()
            .add_cmd(
                BasicCmdBuilder::new()
                    .set_executable("echo")
                    .add_arg("echo")
                    .add_arg("Hallo\nAbc\n123\nAbc123")
            ).add_cmd(
            BasicCmdBuilder::new()
                .set_executable("grep")
                .add_arg("grep")
                .add_arg("-i")
                .add_arg("abc"))
            .add_cmd(
                BasicCmdBuilder::new()
                    .set_executable("wc")
                    .add_arg("wc")
                    .add_arg("-l")
            ).build();

        execute_piped_cmd_chain(&cmd_chain);
    }
}
