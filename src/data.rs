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

use std::ffi::CString;
use crate::libc_util::{construct_libc_cstring, construct_libc_cstring_arr};

pub trait Builder<To>  {
    fn build(self) -> To;
}

#[derive(Debug)]
pub struct BasicCmd {
    executable: String,
    args: Vec<String>,
    in_red_path: Option<String>,
    out_red_path: Option<String>,
    is_first: bool,
    is_last: bool,
}

impl BasicCmd {

    pub fn executable(&self) -> &str {
        &self.executable
    }
    pub fn args(&self) -> &Vec<String> {
        &self.args
    }
    pub fn in_red_path(&self) -> &Option<String> {
        &self.in_red_path
    }
    pub fn out_red_path(&self) -> &Option<String> {
        &self.out_red_path
    }

    pub fn is_first(&self) -> bool {
        self.is_first
    }
    pub fn is_last(&self) -> bool {
        self.is_last
    }
    pub fn is_in_middle(&self) -> bool {
        !self.is_first && !self.is_last
    }

    /// Constructs the null-terminated argv-array on the heap.
    /// Memory must be freed theoretically in order to have proper
    /// memory management but because the address space content is
    /// replaced after "exec()" you don't have to free it in
    /// case of successful exec().
    pub fn args_to_c_argv(&self) -> *const *const libc::c_char {
        let argv: *mut *mut libc::c_char = construct_libc_cstring_arr(self.args.len(), true);

        for i in 0..self.args.len() {
            let arg = &self.args[i];
            let c_string: *mut libc::c_char = construct_libc_cstring(arg);
            unsafe {
                *argv.offset(i as isize) = c_string;
            }
        }

        argv as *const *const libc::c_char
    }

    pub fn executable_cstring(&self) -> CString {
        CString::new(self.executable.clone()).unwrap()
    }

    pub fn out_red_path_cstring(&self) -> Option<CString> {
        self.out_red_path.clone().map(|x| CString::new(x).unwrap())
    }

    pub fn in_red_path_cstring(&self) -> Option<CString> {
        self.in_red_path.clone().map(|x| CString::new(x).unwrap())
    }
}

#[derive(Debug)]
pub struct BasicCmdBuilder {
    executable: Option<String>,
    args: Vec<String>,
    input_redirect_path: Option<String>,
    output_redirect_path: Option<String>,
    is_first: bool,
    is_last: bool,
}

impl BasicCmdBuilder {

    pub fn new() -> Self {
        BasicCmdBuilder {
            executable: None,
            args: vec![],
            input_redirect_path: None,
            output_redirect_path: None,
            is_first: false,
            is_last: false,
        }
    }

    pub fn set_executable(mut self, executable: &str) -> Self {
        self.executable.replace(executable.to_string());
        self
    }
    pub fn add_arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }
    pub fn set_input_redirect_path(mut self, input_redirect_path: &str) -> Self {
        self.input_redirect_path.replace(input_redirect_path.to_string());
        self
    }
    pub fn set_output_redirect_path(mut self, output_redirect_path: &str) -> Self {
        self.output_redirect_path.replace(output_redirect_path.to_string());
        self
    }
    // it's intentionally that this doesn't return self
    fn set_is_first(&mut self, is_first: bool) {
        self.is_first = is_first;
    }

    // it's intentionally that this doesn't return self
    fn set_is_last(&mut self, is_last: bool) {
        self.is_last = is_last;
    }
}

impl Builder<BasicCmd> for BasicCmdBuilder {
    fn build(self) -> BasicCmd {
        BasicCmd {
            executable: self.executable.expect("Must have value"),
            args: self.args,
            in_red_path: self.input_redirect_path,
            out_red_path: self.output_redirect_path,
            is_first: self.is_first,
            is_last: self.is_last,
        }
    }
}

#[derive(Debug)]
pub struct CmdChain {
    // TODO implement in future
    background: bool,
    cmds: Vec<BasicCmd>,
}

impl CmdChain {

    pub fn new(background: bool,
               cmds: Vec<BasicCmd>) -> Self {
        Self { background, cmds }
    }

    pub fn background(&self) -> bool {
        self.background
    }

    pub fn cmds(&self) -> &Vec<BasicCmd> {
        &self.cmds
    }

    pub fn length(&self) -> usize {
        self.cmds.len()
    }
}

#[derive(Debug)]
pub struct CmdChainBuilder {
    background: bool,
    cmds: Vec<BasicCmdBuilder>,
}

impl CmdChainBuilder {

    pub fn new() -> Self {
        CmdChainBuilder {
            background: false,
            cmds: vec![]
        }
    }

    pub fn set_background(mut self, background: bool) -> Self {
        self.background = background;
        self
    }

    pub fn add_cmd(mut self, cmd: BasicCmdBuilder) -> Self {
        self.cmds.push(cmd);
        self
    }
}

impl Builder<CmdChain> for CmdChainBuilder {
    fn build(mut self) -> CmdChain {
        let len = self.cmds.len();
        for i in 0..len {
            let cmd = &mut self.cmds[i];
            cmd.set_is_first(i == 0);
            cmd.set_is_last(i + 1 == len);
        }
        CmdChain {
            background: self.background,
            cmds: self.cmds.into_iter()
                .map(|cmd| cmd.build())
                .collect()
        }
    }
}

/*#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foobar() {

    }

}*/
