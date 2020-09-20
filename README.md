# unix_exec_piper: A library written in Rust that execs multiple commands and connects them with pipes.

## TL;DR;
My library basically does the functionality that happens when you type `cat file.txt | grep -i | wc -l` 
into a shell like bash. **unix_exec_piper** does no parsing, but only the actual execution and connection
between the (child) processes via **unix pipes**.

Important main parts of the library are `pipe.rs` and `lib.rs :: execute_piped_cmd_chain()`.

**The main purpose of this library is educational and to show people who are interested in this how it's done.**

You might build a shell around this library (if it gets more functionality in the future).

## Basics you need to know
Please make yourself familiar with the `UNIX/Poxix` concepts:
- `pipe()`
- `fork()`
- `file descriptors` and *"Everything is a file"*
- `exec()`

## Supported features
- Creating pipes between processes where `STDOUT` of one process gets connected with 
`STDIN` of the next process. \
  (`$ cat file.txt | grep -i | wc -l`) 
- I/O redirection into files \
  `$ cat < file.txt | grep -i | wc -l > out.txt`

## (not yet) supported
- I/O redirection with `STDERR`
- a lot of other redirection primitives listed here: https://tldp.org/LDP/abs/html/io-redirection.html

## Basic idea
The parent process loops `n` times (for `n` commands) and creates `n-1` `Pipe`s. Therefore `n` child processes
are created through `fork()`. Each child process has two variables in its address space:

    let mut pipe_to_current: Option<Pipe>;
    let mut pipe_to_next: Option<Pipe>;

Pipes communicates across process boundaries in the following way:

    child process 0    child process 1    child process n
    _______________    _______________    _________
    | cat foo.txt |    | grep -i abc |    | wc -l |
    ---------------    ---------------    ---------
                ^        ^         ^        ^
          WRITE |--------|  R / W  |--------| READ
          END               E   E             END
                       (current child)
            -Pipe to Current-   -Pipe to Next-
            
Each process uses `pipe_to_current` (if present) as "read end" (as it's `stdin`) and
`pipe_to_current` (if present) as "write end" 
(duplicate it's `STDOUT` `file descriptor` into the write end of the pipe).
