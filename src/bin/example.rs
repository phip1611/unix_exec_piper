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

use unix_exec_piper::{execute_piped_cmd_chain, CmdChainBuilder, BasicCmdBuilder, Builder, update_process_states};

/// Please note: My experience showed that somehow output redirection into a file make
/// problems when executed from the IDE (CLion), at least when the path is inside src. Don't
/// know what's happening there.
/// If executing from IDE just use a path outside of src.
fn main() {
    // Example one: 'cat < src/bin/testfile_65kb.txt | cat | cat > foobar.txt'

    // Test with the "testfile_65kb.txt". If piping between processes
    // does not stuck, all FDs are closed properly.
    let cmd_chain = CmdChainBuilder::new()
        .add_cmd(
            BasicCmdBuilder::new()
                .set_executable("cat")
                .add_arg("cat")
                //.add_arg("src/bin/testfile_65kb.txt")
                .set_input_redirect_path("src/bin/testfile_65kb.txt")
        ).add_cmd(
        BasicCmdBuilder::new()
            .set_executable("cat")
            .add_arg("cat"))
        .add_cmd(
        BasicCmdBuilder::new()
            .set_executable("cat")
            .add_arg("cat")
            .set_output_redirect_path("foobar.txt")
    ).build();
    execute_piped_cmd_chain(&cmd_chain);

    println!();
    println!("############################################################");
    println!();

    // Example two: 'ls -l | grep -i a &'
    // (with waiting in background)
    let cmd_chain = CmdChainBuilder::new()
        .add_cmd(
            BasicCmdBuilder::new()
                .set_executable("ls")
                .add_arg("ls")
                .add_arg("-l")
        )
        .add_cmd(
            BasicCmdBuilder::new()
                .set_executable("grep")
                .add_arg("grep")
                .add_arg("-i")
                .add_arg("a")
        )
        .set_background(true)
        .build();
    let mut state = execute_piped_cmd_chain(&cmd_chain);
    println!("Process states after dispatch: {:#?}", state);
    while !update_process_states(&mut state, true) {
        /*
         * Example two wait non-blocking. This check could be done for example
         * in a shell everytime the user presses 'enter'
         */
    }
    println!("Process states after finished: {:#?}", state);
}