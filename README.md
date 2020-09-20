# unix_exec_piper: A library written in Rust that exec's multiple commands and connects them with pipes.

## TL;DR;
My library basically does the functionality that happens when you type `cat file.txt | grep -i | wc -l` 
into a shell like bash. **unix_exec_piper** does no parsing and only the actual execution and connection
between the (child) processes via **unix pipes**.

Main parts in library are `pipe.rs` and `lib.rs :: execute_piped_cmd_chain()`.

**The main purpose of this library is educational and to show people who are interested in this how it's done.**

You might build a shell around this library (if it gets more functionality in the future).

## Supported features
- Creating pipes between processes whereas STDOUT of one process gets connected with 
STDIN of the next process. (`$ cat file.txt | grep -i | wc -l`) 
- I/O redirection into files `$ cat < file.txt | grep -i | wc -l > out.txt`

## (not yet) supported
- I/O redirection with stderr
- a lot of other redirection primitives listed here: https://tldp.org/LDP/abs/html/io-redirection.html
