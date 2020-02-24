use crate::context::Context;
use crate::alloc::alloc::{
    alloc,
    dealloc,
    Layout,
};
use crate::consts::*;
use riscv::register::satp;
use alloc::boxed::Box;

pub struct KernelStack(usize);

impl KernelStack {
    pub fn new() -> Self {
        let bottom = unsafe {
            alloc(Layout::from_size_align(KERNEL_STACK_SIZE, KERNEL_STACK_SIZE).unwrap()) as usize
        };
        KernelStack(bottom)
    }
    pub fn new_empty() -> Self {
        KernelStack(0)
    }
    pub fn top(&self) -> usize {
        self.0 + KERNEL_STACK_SIZE
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        if self.0 != 0 {
            unsafe {
                dealloc(
                    self.0 as _,
                    Layout::from_size_align(KERNEL_STACK_SIZE, KERNEL_STACK_SIZE).unwrap(),
                );
            }
        }
    }
}

pub struct Thread {
    // 线程的状态
    pub context: Context,
    // 线程的栈
    pub kstack: KernelStack,
}

impl Thread {
    pub fn switch_to(&mut self, target: &mut Thread) {
        unsafe { self.context.switch(&mut target.context); }
    }
    pub fn get_boot_thread() -> Box<Thread> {
        Box::new(Thread {
            context: Context::null(),
            kstack: KernelStack::new_empty(),
        })
    }
    pub fn new_kernel(entry: usize) -> Box<Thread> {
        unsafe {
            let kstack_ = KernelStack::new();
            Box::new(Thread {
                // 内核线程共享内核资源，因此用目前的 satp 即可
                context: Context::new_kernel_thread(entry, kstack_.top(), satp::read().bits()), kstack: kstack_,
            })
        }
    }
    // 为线程传入初始参数
    pub fn append_initial_arguments(&self, args: [usize; 3]) {
        unsafe { self.context.append_initial_arguments(args); }
    }
}