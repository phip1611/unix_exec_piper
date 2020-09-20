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

/// Abstraction over UNIX pipe. This Pipe abstraction is specific
/// to the case of connecting once process' STDOUT with next
/// process' STDIN. A parent process is creating n childs and
/// creates n-1 pipes. Each child process knows it's
/// optional pipe_to_current and it's optional pipe_to_next.
///
/// Please note that each pipe object exists in each address space
/// of each child. See fork() for more information:
/// https://man7.org/linux/man-pages/man2/fork.2.html
///
/// ```
/// /*
/// _______________    _______________    _________
/// | cat foo.txt |    | grep -i abc |    | wc -l |
/// ---------------    ---------------    ---------
///             ^        ^         ^        ^
///       WRITE |--------|  R / W  |--------| READ
///       END               E   E             END
///                    (current child)
///         -Pipe to Current-   -Pipe to Next-`
/// */
/// ```

/// Index in the `fd[i32; 2]`-array.
/// See https://man7.org/linux/man-pages/man2/pipe.2.html
#[derive(Debug, Copy, Clone)]
pub enum PipeEnd {
    Read = 0,
    Write = 1,
}

/* _______________    _______________    _________
 * | cat foo.txt |    | grep -i abc |    | wc -l |
 * ---------------    ---------------    ---------
 *             ^        ^         ^        ^
 *       WRITE |--------|  R / W  |--------| READ
 *       END               E   E             END
 *                    (current child)
 *         -Pipe to Current-   -Pipe to Next-
 */
/**
 * Abstraction over UNIX pipe for the specific case here with
 * stdin/stdout redirection between processes. The typical flow
 * is that a Pipe is created, the program is forked and
 * that one process marks it's part of the Pipe as READ
 * while the other process marks it's part of the Pipe
 * as WRITE.
 *
 * Each Pipe object will exists per address space, because
 * we create a child process for each command to be executed.
 *
 * Each pipe connects two processes. Each process has
 * access to "pipe_to_current" and "pipe_to_next".
 * First one is used as READ-end while the latter one
 * is used as WRITE-end.
 */
#[derive(Debug)]
pub struct Pipe {
    /// The file descriptors.
    fds: [i32; 2],
    /// If this Pipe has been marked already as either READ
    /// or WRITE end in this address space.
    locked: bool,
    /// If read fd has already been closed.
    read_closed: bool,
    /// If write fd has already been closed.
    write_closed: bool,
}

impl Pipe {

    pub fn new() -> Self {
        let mut fds: [libc::c_int; 2] = [0; 2];
        let res = unsafe { libc::pipe(fds.as_mut_ptr()) };
        if res == -1 { panic!("Pipe creation failed!") }
        Self {
            fds,
            locked: false,
            read_closed: false,
            write_closed: false
        }
    }

    /// Marks and locks the Pipe in the current address space
    /// as read end.
    pub fn as_read_end(&mut self) {
        // This operation should/must be done only once per address space!
        if self.locked { panic!("Pipe is already locked!") }
        self.locked = true;
        self.close_pipe_end(PipeEnd::Write);
        self.write_closed = true;
        self.connect_pipe_end(PipeEnd::Read, libc::STDIN_FILENO);
    }

    /// Marks and locks the Pipe in the current address space
    /// as write end.
    pub fn as_write_end(&mut self) {
        // This operation should/must be done only once per address space!
        if self.locked { panic!("Pipe is already locked!") }
        self.locked = true;
        self.close_pipe_end(PipeEnd::Read);
        self.read_closed = true;
        self.connect_pipe_end(PipeEnd::Write, libc::STDOUT_FILENO);
    }

    /// Connects a pipe end with another file descriptor.
    fn connect_pipe_end(&mut self, pe: PipeEnd, file_no: libc::c_int) {
        assert!(file_no == libc::STDIN_FILENO || file_no == libc::STDOUT_FILENO);

        let res = unsafe { libc::dup2(self.fds[pe as usize], file_no) };
        if res == -1 {
            panic!("Connecting {:?}-end of Pipe with {} failed! {}", pe, file_no, errno::errno())
        }
    }

    /// Closes the file descriptor of a pipe end.
    fn close_pipe_end(&self, pe: PipeEnd) {
        let res = unsafe { libc::close(self.fds[pe as usize]) };
        if res == -1 { panic!("Closing {:?}-end of pipe failed! {}", pe, errno::errno()) }
    }

    /// A parent doesn't uses the pipes. It just creates the objects and make
    /// sure they are transferred into the childs (via fork(). After a child
    /// process started and got it's Pipe objects, the parent MUST close
    /// it's FDs in order to prevent deadlocks.
    pub fn parent_close_all(&mut self) {
        if !self.write_closed {
            self.close_pipe_end(PipeEnd::Write);
            self.write_closed = true;
        }
        if !self.read_closed {
            self.close_pipe_end(PipeEnd::Read);
            self.read_closed = true;
        }
    }

}

impl Drop for Pipe {
    /// Makes sure all FD's are closed when Pipe is dropped.
    fn drop(&mut self) {
        // I think this should never be really needed because
        // the parent process calls "parent_close_all" anyway.
        // But it's nice for a proper shutdown of the child processes
        // when they exit.
        if !self.write_closed {
            self.close_pipe_end(PipeEnd::Write)
        }
        if !self.read_closed {
            self.close_pipe_end(PipeEnd::Read)
        }
    }
}
