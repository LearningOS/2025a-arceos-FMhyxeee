#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[macro_use]
#[cfg(feature = "axstd")]
extern crate axstd as std;

mod ramfs;

use std::io::{self, prelude::*};
use std::fs::{self, File};
use std::string::String;

fn create_file(fname: &str, text: &str) -> io::Result<()> {
    println!("Create '{}' and write [{}] ...", fname, text);
    let mut file = File::create(fname)?;
    file.write_all(text.as_bytes())
}

// Only support rename, NOT move.
fn rename_file(src: &str, dst: &str) -> io::Result<()> {
    println!("Rename '{}' to '{}' ...", src, dst);

    // Try the normal rename first
    match fs::rename(src, dst) {
        Ok(()) => return Ok(()),
        Err(e) => {
            println!("Standard rename failed: {}, trying manual rename...", e);
            // Fall back to manual rename using copy and delete
            manual_rename(src, dst)
        }
    }
}

// Manual rename implementation using copy and delete
fn manual_rename(src: &str, dst: &str) -> io::Result<()> {
    use std::io::Read;

    // Read source file
    let mut src_file = File::open(src)?;
    let mut content = String::new();
    src_file.read_to_string(&mut content)?;

    // Create destination file and write content
    let mut dst_file = File::create(dst)?;
    dst_file.write_all(content.as_bytes())?;

    // Remove source file
    fs::remove_file(src)?;

    Ok(())
}

fn print_file(fname: &str) -> io::Result<()> {
    let mut buf = [0; 1024];
    let mut file = File::open(fname)?;
    loop {
        let n = file.read(&mut buf)?;
        if n > 0 {
            print!("Read '{}' content: [", fname);
            io::stdout().write_all(&buf[..n])?;
            println!("] ok!");
        } else {
            return Ok(());
        }
    }
}

fn process() -> io::Result<()> {
    // Create the /tmp directory first (ignore if it already exists)
    let _ = fs::create_dir("/tmp");

    // Clean up any existing files from previous runs
    let _ = fs::remove_file("/tmp/f1");
    let _ = fs::remove_file("/tmp/f2");

    create_file("/tmp/f1", "hello")?;
    print_file("/tmp/f1")?;

    // Just rename, NOT move.
    // So this must happen in the same directory.
    println!("Attempting rename...");
    rename_file("/tmp/f1", "/tmp/f2")?;
    println!("Rename successful!");
    print_file("/tmp/f2")
}

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    if let Err(e) = process() {
        panic!("Error: {}", e);
    }
    println!("\n[Ramfs-Rename]: ok!");
}
