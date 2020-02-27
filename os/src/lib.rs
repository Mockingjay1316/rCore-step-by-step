#![no_std]
#![feature(asm)]
#![feature(global_asm)]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]

#[macro_use]
mod io;

mod init;
mod lang_items;
mod sbi;
mod interrupt;
mod context;
mod timer;
mod consts;
mod memory;
mod process;
mod syscall;

extern crate alloc;