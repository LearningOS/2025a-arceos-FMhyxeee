#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
extern crate axstd as std;
extern crate alloc;

#[macro_use]
extern crate axlog;

mod task;
mod syscall;
mod loader;

use axstd::io::{self, Read};
use axhal::paging::MappingFlags;
use axhal::arch::UspaceContext;
use axhal::mem::VirtAddr;
use axsync::Mutex;
use alloc::sync::Arc;
use alloc::string::String;
use alloc::collections::BTreeMap;
use axmm::AddrSpace;
use loader::load_user_app;

const USER_STACK_SIZE: usize = 0x10000;
const KERNEL_STACK_SIZE: usize = 0x40000; // 256 KiB

#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    // A new address space for user app.
    let mut uspace = axmm::new_user_aspace().unwrap();

    // Create a simple test app in ramfs
    use std::fs::{self, File};
    use std::io::Write;

    // Create a simple ELF-like binary in ramfs
    let _ = fs::create_dir("/tmp");
    let mut test_file = File::create("/tmp/test_app").unwrap();
    // Write some simple test data (this would normally be an ELF binary)
    test_file.write_all(b"hello, arceos!").unwrap();
    test_file.sync_all().unwrap();
    drop(test_file);

    // For now, let's just read the file back to verify ramfs works
    let mut file = File::open("/tmp/test_app").unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    ax_println!("Read back content: {}", content);

    // TODO: Load actual user app when available
    // let entry = match load_user_app("/tmp/test_app", &mut uspace) {
    //     Ok(e) => e,
    //     Err(err) => panic!("Cannot load app! {:?}", err),
    // };
    let entry = 0x1000; // dummy entry point
    ax_println!("entry: {:#x}", entry);

    // Init user stack.
    let ustack_top = init_user_stack(&mut uspace, true).unwrap();
    ax_println!("New user address space: {:#x?}", uspace);

    // Let's kick off the user process.
    let user_task = task::spawn_user_task(
        Arc::new(Mutex::new(uspace)),
        UspaceContext::new(entry, ustack_top),
    );

    // Wait for user process to exit ...
    let exit_code = user_task.join();
    ax_println!("monolithic kernel exit [{:?}] normally!", exit_code);
}

fn init_user_stack(uspace: &mut AddrSpace, populating: bool) -> io::Result<VirtAddr> {
    let ustack_top = uspace.end();
    let ustack_vaddr = ustack_top - crate::USER_STACK_SIZE;
    ax_println!(
        "Mapping user stack: {:#x?} -> {:#x?}",
        ustack_vaddr, ustack_top
    );
    uspace.map_alloc(
        ustack_vaddr,
        crate::USER_STACK_SIZE,
        MappingFlags::READ | MappingFlags::WRITE | MappingFlags::USER,
        populating,
    ).unwrap();

    let app_name = "hello";
    let av = BTreeMap::new();
    let (stack_data, ustack_pointer) = kernel_elf_parser::get_app_stack_region(
        &[String::from(app_name)],
        &[],
        &av,
        ustack_vaddr,
        crate::USER_STACK_SIZE,
    );
    uspace.write(VirtAddr::from_usize(ustack_pointer), stack_data.as_slice())?;

    Ok(ustack_pointer.into())
}
