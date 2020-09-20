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

use unix_exec_piper::execute_piped_cmd_chain;
use unix_exec_piper::data::{CmdChainBuilder, BasicCmdBuilder, Builder};

fn main() {
    // Test with the "testfile_65kb.txt". If piping between processes
    // does not stuck, all FDs are closed properly.
    let cmd_chain = CmdChainBuilder::new()
        .add_cmd(
            BasicCmdBuilder::new()
                .set_executable("cat")
                .add_arg("cat")
                .add_arg("src/bin/testfile_65kb.txt")
        ).add_cmd(
        BasicCmdBuilder::new()
            .set_executable("cat")
            .add_arg("cat"))
        .add_cmd(
        BasicCmdBuilder::new()
            .set_executable("cat")
            .add_arg("cat")
    ).build();
    execute_piped_cmd_chain(&cmd_chain);

    // Test input output redirect with files
    let cmd_chain = CmdChainBuilder::new()
        .add_cmd(
            BasicCmdBuilder::new()
                .set_executable("cat")
                .add_arg("cat")
                .set_input_redirect_path("src/bin/testfile_65kb.txt")
        )
        .add_cmd(
            BasicCmdBuilder::new()
                .set_executable("cat")
                .add_arg("cat")
        ).add_cmd(
        BasicCmdBuilder::new()
            .set_executable("cat")
            .add_arg("cat")
            .set_output_redirect_path("src/bin/out.txt"))
        .build();
    execute_piped_cmd_chain(&cmd_chain);
}