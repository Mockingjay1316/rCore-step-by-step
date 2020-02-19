use riscv::register::{
    scause,
    sepc,
    stvec,
    sscratch,
    sstatus
};
use crate::context::TrapFrame;

global_asm!(include_str!("trap/trap.asm"));

pub fn init() {
    unsafe {
        extern "C" {
            // 中断处理总入口
            fn __alltraps();
        }
        // 经过上面的分析，由于现在是在内核态
        // 我们要把 sscratch 初始化为 0
        sscratch::write(0);
        // 仍使用 Direct 模式
        // 将中断处理总入口设置为 __alltraps
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
        // 设置 sstatus 的 SIE 位
        sstatus::set_sie();
    }
    println!("++++ setup interrupt! ++++");
}

#[no_mangle]
fn rust_trap(tf: &mut TrapFrame) {
    // let cause = tf.scause.bits();
    // let epc = tf.sepc;
    // println!("trap: cause: {:?}, epc: 0x{:#x}", cause, epc);
    println!("rust_trap!");
    tf.sepc += 2;
}