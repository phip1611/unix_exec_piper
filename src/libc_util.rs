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

/// Utility functions on top of libc.
/// I've chosen to use `*mut libc::c_char"` rather than `std::ffi::CStr`
/// because of educational purposes, to gain more experience, and just
/// for fun.

/// Constructs an array of C strings aka. array of `*mut libc::c_char"` on
/// the heap. Allocates memory. Memory must be freed manually somewhere in
/// order to have proper memory management.
/// I've chosen to use `*mut libc::c_char"` rather than `std::ffi::CStr`
/// because of educational purposes, to gain more experience, and just
/// for fun.
pub fn construct_libc_cstring_arr(elements_count: usize, null_terminated: bool) -> *mut *mut libc::c_char {
    let elements = if null_terminated { elements_count + 1 } else { elements_count };
    let ptr_size = get_c_ptr_size();
    let argv: *mut *mut libc::c_char;
    unsafe {
        // allocate memory for array of pointers
        // we use calloc for null terminated array
        argv = std::mem::transmute(
            libc::calloc(ptr_size, elements)
        );
    }
    argv
}

/// Allocates memory on the heap and constructs a null-terminated C string
/// from a Rust string.
/// I've chosen to use `*mut libc::c_char"` rather than `std::ffi::CStr`
/// because of educational purposes, to gain more experience, and just
/// for fun.
pub fn construct_libc_cstring(string: &str) -> *mut libc::c_char {
    let char_size = 1; // 1 byte
    let c_string: *mut libc::c_char;
    unsafe {
        c_string = std::mem::transmute(
            // + 1: null terminated
            libc::malloc( char_size * (string.len() + 1))
        );
    }

    let chars = string.chars().collect::<Vec<char>>();
    for i in 0..chars.len() {
        unsafe {
            *c_string.offset(i as isize) = chars[i] as libc::c_char;
        }
    }

    // null terminated
    unsafe {
        *c_string.offset(string.len() as isize) = 0;
    }

    c_string
}

// we don't have "sizeof()" in Rust like we have it in C/C++.
// Therefore I use this compile time ("const") function to calculate
// the size.
/// Returns the size of a `* libc::c_char`-Pointer.
/// TODO: Make this function `const` as soon as
///  `std::mem::size_of_val` supports const.
/// This is done at runtime for now which is not
/// ideal.
fn get_c_ptr_size() -> usize {
    let example_char: libc::c_char = 5;
    let example_char_ptr = &example_char;
    // TODO make this 'const fn' when size_of_val() supports it
    let c_ptr_size = std::mem::size_of_val(&example_char_ptr);
    c_ptr_size
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_construct_libc_cstring() {
        let input = String::from("Hello World!");

        // Check against std::ffi::&CStr
        let c_str: &CStr = unsafe { CStr::from_ptr(construct_libc_cstring(&input)) };
        println!("expected: '{}'", input);
        println!("actual:   '{}'", c_str.to_str().unwrap().to_owned());
        assert_eq!(unsafe {libc::strlen(c_str.as_ptr())}, input.len());
    }

    #[test]
    fn test_construct_libc_cstring_arr() {
        let elem_count = 2;
        let is_null_terminated = true;
        let arr = construct_libc_cstring_arr(2, is_null_terminated);
        unsafe {
            *arr.offset(0) = construct_libc_cstring("First");
            *arr.offset(1) = construct_libc_cstring("Second");
        }

        let input = String::from("Hello World!");

        // Check against std::ffi::&CStr to see if we did correct work
        let c_str1: &CStr = unsafe { CStr::from_ptr(*arr.offset(0)) };
        let c_str2: &CStr = unsafe { CStr::from_ptr(*arr.offset(1)) };
        println!("expected 1: '{}'", "First");
        println!("actual 1:   '{}'", c_str1.to_str().unwrap().to_owned());
        println!("expected 2: '{}'", "Second");
        println!("actual 2:   '{}'", c_str2.to_str().unwrap().to_owned());


        // check has null terminated end
        // .as_ref(): Returns `None` if the pointer is null

        let to = if is_null_terminated {
            elem_count + 1
        } else {
            elem_count
        };
        for i in 0..to {
            let ptr_val = unsafe { *(arr.offset(i) as * const libc::c_int) };
            if is_null_terminated && i == elem_count {
                // expect null pointer because we are null terminated here
                assert_eq!(0, ptr_val);
            } else {
                assert_ne!(0, ptr_val);
            }
        }
    }


}
