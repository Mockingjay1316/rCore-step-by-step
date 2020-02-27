use riscv::register::{
    sstatus::Sstatus,
    scause::Scause,
};
use riscv::register::sstatus;
use core::mem::zeroed;

#[repr(C)]      // Disable struct reposition
// #[derive(Debug)]
pub struct TrapFrame {
    pub x: [usize; 32],     // General registers
    pub sstatus: Sstatus,   // Supervisor Status Register
    pub sepc: usize,        // Supervisor exception program counter
    pub stval: usize,       // Supervisor trap value
    pub scause: Scause,     // Scause register: record the cause of exception/interrupt/trap
}

extern "C" {
	fn __trapret();
}

#[repr(C)]
pub struct ContextContent {
    pub ra: usize,
    satp: usize,
    s: [usize; 12],
    tf: TrapFrame,
}

impl ContextContent {
    // 为一个新内核线程构造栈上的初始状态信息
    // 其入口点地址为 entry ，其内核栈栈顶地址为 kstack_top ，其页表为 satp
    fn new_kernel_thread(
        entry: usize,
        kstack_top: usize,
        satp: usize,
        ) -> ContextContent {

        let mut content = ContextContent {
            ra: __trapret as usize,
            satp,
            s: [0; 12],
            tf: {
                let mut tf: TrapFrame = unsafe { zeroed() };
                tf.x[2] = kstack_top;
                tf.sepc = entry;
                tf.sstatus = sstatus::read();
                tf.sstatus.set_spp(sstatus::SPP::Supervisor);
                tf.sstatus.set_spie(true);
                tf.sstatus.set_sie(false);
                tf
            }
        };
        content
    }
    fn new_user_thread(
        entry: usize,
        ustack_top: usize,
        satp: usize
        ) -> Self {
        ContextContent {
            ra: __trapret as usize,
            satp,
            s: [0; 12],
            tf: {
                let mut tf: TrapFrame = unsafe { zeroed() };
                // 利用 __trapret 返回后设置为用户栈
                tf.x[2] = ustack_top;
                // 设置 sepc 从而在 sret 之后跳转到用户程序入口点
                tf.sepc = entry;
                tf.sstatus = sstatus::read();
                tf.sstatus.set_spie(true);
                tf.sstatus.set_sie(false);
                // 设置 sstatus 的 spp 字段为 User
                // 从而在 sret 之后 CPU 的特权级将变为 U Mode
                tf.sstatus.set_spp(sstatus::SPP::User);
                tf
            }
        }
    }
    // 将自身压到栈上，并返回 Context
    unsafe fn push_at(self, stack_top: usize) -> Context {
        let ptr = (stack_top as *mut ContextContent).sub(1);
        *ptr = self;
        Context { content_addr: ptr as usize }
    }
}

#[repr(C)]
pub struct Context {
    pub content_addr: usize,
}

impl Context {
    #[naked]
    #[inline(never)]
    pub unsafe extern "C" fn switch(&mut self, _target: &mut Context) {
        asm!(include_str!("process/switch.asm") :::: "volatile");
    }
    pub fn null() -> Context {
        Context { content_addr: 0, }
    }

    pub unsafe fn new_kernel_thread(
        entry: usize,
        kstack_top: usize,
        satp: usize
        ) -> Context {
        ContextContent::new_kernel_thread(entry, kstack_top, satp).push_at(kstack_top)
    }
    pub unsafe fn new_user_thread(
        entry: usize,
        ustack_top: usize,
        kstack_top: usize,
        satp: usize
        ) -> Self {
        // 压到内核栈
        ContextContent::new_user_thread(entry, ustack_top, satp).push_at(kstack_top)
    }
    pub unsafe fn append_initial_arguments(&self, args: [usize; 3]) {
        let context_content = &mut *(self.content_addr as *mut ContextContent);
        context_content.tf.x[10] = args[0];
        context_content.tf.x[11] = args[1];
        context_content.tf.x[12] = args[2];
    }
}