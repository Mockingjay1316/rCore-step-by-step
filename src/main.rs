#![no_std]
use core::panic::PanicInfo;

fn main() {
    // println!("Hello, world!");
    // println! require OS support.
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}
