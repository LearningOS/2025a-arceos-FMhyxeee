#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::{println, print};

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    // Output with red color using ANSI escape codes
    print!("\x1b[31m[WithColor]: Hello, Arceos!\x1b[0m\n");
}
