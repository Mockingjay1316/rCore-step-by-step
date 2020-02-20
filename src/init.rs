use crate::consts::*;
use crate::memory::{
    alloc_frame,
    dealloc_frame
};

global_asm!(include_str!("boot/entry64.asm"));

#[no_mangle]
pub extern "C" fn rust_main() -> ! {
    // get addr location from extern
    extern "C" {
        fn _start();
        fn bootstacktop();
        fn end();
    }
    println!("_start vaddr = 0x{:x}", _start as usize);
    println!("bootstacktop vaddr = 0x{:x}", bootstacktop as usize);
    println!(
        "free physical memory paddr = [{:#x}, {:#x})",
        end as usize - KERNEL_BEGIN_VADDR + KERNEL_BEGIN_PADDR,
        PHYSICAL_MEMORY_END
    );
    println!(
        "free physical memory ppn = [{:#x}, {:#x})",
        ((end as usize - KERNEL_BEGIN_VADDR + KERNEL_BEGIN_PADDR) >> 12) + 1,
        PHYSICAL_MEMORY_END >> 12
);
    crate::interrupt::init();
    crate::memory::init(
        ((end as usize - KERNEL_BEGIN_VADDR + KERNEL_BEGIN_PADDR) >> 12) + 1,
        PHYSICAL_MEMORY_END >> 12
    );
    frame_allocating_test();
    crate::timer::init();
    unsafe {
        asm!("ebreak"::::"volatile");
    }
    panic!("end of rust_main");
}

fn frame_allocating_test() {
    println!("alloc {:x?}", alloc_frame());
    let f = alloc_frame();
    println!("alloc {:x?}", f);
    println!("alloc {:x?}", alloc_frame());
    println!("dealloc {:x?}", f);
    dealloc_frame(f.unwrap());
    println!("alloc {:x?}", alloc_frame());
    println!("alloc {:x?}", alloc_frame());
}