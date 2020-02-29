use riscv::register::{
    scause::{
        self,
        Trap,
        Exception,
        Interrupt
    },
    sepc,
    stvec,
    sscratch,
    sstatus,
    sie
};
use crate::timer::{
    TICKS,
    clock_set_next_event
};
use crate::context::TrapFrame;
use crate::process::tick;
use crate::memory::access_pa_via_va;

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
        // enable external interrupt
        sie::set_sext();

        // closed by OpenSBI, so we open them manually
        // see https://github.com/rcore-os/rCore/blob/54fddfbe1d402ac1fafd9d58a0bd4f6a8dd99ece/kernel/src/arch/riscv32/board/virt/mod.rs#L4
        init_external_interrupt();
        enable_serial_interrupt();
    }
    println!("++++ setup interrupt! ++++");
}

#[no_mangle]
fn rust_trap(tf: &mut TrapFrame) {
    // let cause = tf.scause.bits();
    // let epc = tf.sepc;
    // println!("trap: cause: {:?}, epc: 0x{:#x}", cause, epc);
    // println!("rust_trap!");
    match tf.scause.cause() {
        // 断点中断
        Trap::Exception(Exception::Breakpoint) => breakpoint(&mut tf.sepc),
        // S态时钟中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => super_timer(),
        Trap::Interrupt(Interrupt::SupervisorExternal) => external(),
        Trap::Exception(Exception::UserEnvCall) => syscall(tf),
        Trap::Exception(Exception::InstructionPageFault) => page_fault(tf),
        Trap::Exception(Exception::LoadPageFault) => page_fault(tf),
        Trap::Exception(Exception::StorePageFault) => page_fault(tf),
        _ => panic!("undefined trap!")
    }
}

fn breakpoint(sepc: &mut usize) {
    println!("a breakpoint set @0x{:x}", sepc);
    *sepc += 2;
}

fn page_fault(tf: &mut TrapFrame) {
    println!("{:?} va = {:#x} instruction = {:#x}", tf.scause.cause(), tf.stval, tf.sepc);
    panic!("page fault!");
}

fn super_timer() {
    // 设置下一次时钟中断触发时间
    clock_set_next_event();
    tick();
    unsafe {
        // 更新时钟中断触发计数
        // 注意由于 TICKS 是 static mut 的
        // 后面会提到，多个线程都能访问这个变量
        // 如果同时进行 +1 操作，会造成计数错误或更多严重bug
        // 因此这是 unsafe 的，不过目前先不用管这个
        TICKS += 1;
        // 每触发 100 次时钟中断将计数清零并输出
        if TICKS == 100 {
            TICKS = 0;
            //println!("* 100 ticks *");
        }
    }
    // 由于一般都是在死循环内触发时钟中断
    // 因此我们同样的指令再执行一次也无妨
    // 因此不必修改 sepc
}

fn external() {
    // 键盘属于一种串口设备，而实际上有很多种外设
    // 这里我们只考虑串口
    let _ = try_serial();
}

fn try_serial() -> bool {
    // 通过 OpenSBI 获取串口输入
    match super::io::getchar_option() {
        Some(ch) => {
            // 将获取到的字符输入标准输入
            if (ch == '\r') {
                crate::fs::stdio::STDIN.push('\n');
            }
            else {
                crate::fs::stdio::STDIN.push(ch);
            }
            true
        },
        None => false
    }
}

fn syscall(tf: &mut TrapFrame) {
    // 返回后跳转到 ecall 下一条指令
    tf.sepc += 4;
    let ret = crate::syscall::syscall(
        tf.x[17],
        [tf.x[10], tf.x[11], tf.x[12]],
        tf
    );
    tf.x[10] = ret as usize;
}

#[inline(always)]
pub fn disable_and_store() -> usize {
    let sstatus: usize;
    unsafe {
        // clear sstatus 的 SIE 标志位禁用异步中断
        // 返回 clear 之前的 sstatus 状态
        asm!("csrci sstatus, 1 << 1" : "=r"(sstatus) ::: "volatile");
    }
    sstatus
}

#[inline(always)]
pub fn restore(flags: usize) {
    unsafe {
        // 将 sstatus 设置为 flags 的值
        asm!("csrs sstatus, $0" :: "r"(flags) :: "volatile");
    }
}

#[inline(always)]
pub fn enable_and_wfi() {
    unsafe {
        // set sstatus 的 SIE 标志位启用异步中断
        // 并通过 wfi 指令等待下一次异步中断的到来
        asm!("csrsi sstatus, 1 << 1; wfi" :::: "volatile");
    }
}

pub unsafe fn init_external_interrupt() {
    let HART0_S_MODE_INTERRUPT_ENABLES: *mut u32 = access_pa_via_va(0x0c00_2080) as *mut u32;
    const SERIAL: u32 = 0xa;
    HART0_S_MODE_INTERRUPT_ENABLES.write_volatile(1 << SERIAL);
}

pub unsafe fn enable_serial_interrupt() {
    let UART16550: *mut u8 = access_pa_via_va(0x10000000) as *mut u8;
    UART16550.add(4).write_volatile(0x0B);
    UART16550.add(1).write_volatile(0x01);
}