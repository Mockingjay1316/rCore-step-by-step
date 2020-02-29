use crate::context::Context;
use crate::alloc::alloc::{
    alloc,
    dealloc,
    Layout,
};
use crate::consts::*;
use riscv::register::satp;
use alloc::boxed::Box;
use super::{ Tid, ExitCode };
use xmas_elf::{
    header,
    program::{ Flags, SegmentData, Type },
    ElfFile,
};
use crate::memory::memory_set::{
    MemorySet,
    handler::ByFrame,
    attr::MemoryAttr,
};
use core::str;

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
    pub wait: Option<Tid>,
}

impl Thread {
    pub fn switch_to(&mut self, target: &mut Thread) {
        unsafe { self.context.switch(&mut target.context); }
    }
    pub fn get_boot_thread() -> Box<Thread> {
        Box::new(Thread {
            context: Context::null(),
            kstack: KernelStack::new_empty(),
            wait: None,
        })
    }
    pub fn new_kernel(entry: usize) -> Box<Thread> {
        unsafe {
            let kstack_ = KernelStack::new();
            Box::new(Thread {
                // 内核线程共享内核资源，因此用目前的 satp 即可
                context: Context::new_kernel_thread(entry, kstack_.top(), satp::read().bits()),
                kstack: kstack_,
                wait: None,
            })
        }
    }
    pub unsafe fn new_user(data: &[u8], wait_thread: Option<Tid>) -> Box<Thread> {
        // 确认合法性
        let elf = ElfFile::new(data).expect("failed to analyse elf!");

        match elf.header.pt2.type_().as_type() {
            header::Type::Executable => {
                println!("it really a executable!");
            },
            header::Type::SharedObject => {
                panic!("shared object is not supported!");
            },
            _ => {
                panic!("unsupported elf type!");
            }
        }
        // 获取入口点
        let entry_addr = elf.header.pt2.entry_point() as usize;
        // 为用户程序创建新的虚拟内存空间
        let mut vm = elf.make_memory_set();

        // 创建用户栈
        let mut ustack_top = {
            // 这里我们将用户栈固定在虚拟内存空间中的某位置
            let (ustack_bottom, ustack_top) = (USER_STACK_OFFSET, USER_STACK_OFFSET + USER_STACK_SIZE);
            // 将用户栈插入虚拟内存空间
            vm.push(
                ustack_bottom,
                ustack_top,
                // 注意这里设置为用户态
                MemoryAttr::new().set_user(),
                ByFrame::new(),
                None,
            );
            ustack_top
        };

        // 创建内核栈
        let kstack = KernelStack::new();

        Box::new(
            Thread {
                context: Context::new_user_thread(entry_addr, ustack_top, kstack.top(), vm.token()),
                kstack: kstack,
                wait: wait_thread,
            }
        )
    }
    // 为线程传入初始参数
    pub fn append_initial_arguments(&self, args: [usize; 3]) {
        unsafe { self.context.append_initial_arguments(args); }
    }
}

#[derive(Clone)]
pub enum Status {
    // 就绪：可以运行，但是要等到 CPU 的资源分配给它
    Ready,
    // 正在运行
    Running(Tid),
    // 睡眠：当前被阻塞，要满足某些条件才能继续运行
    Sleeping,
    // 退出：该线程执行完毕并退出
    Exited(ExitCode),
}

trait ElfExt {
    fn make_memory_set(&self) -> MemorySet;
}
// 给一个用户程序的ELF可执行文件创建虚拟内存空间
impl ElfExt for ElfFile<'_> {
    fn make_memory_set(&self) -> MemorySet {
        // MemorySet::new()的实现中已经映射了内核各数据、代码段，以及物理内存段
        // 于是我们只需接下来映射用户程序各段即可
        let mut memory_set = MemorySet::new();
        for ph in self.program_iter() {
            // 遍历各段并依次尝试插入 memory_set
            if ph.get_type() != Ok(Type::Load) {
                continue;
            }
            let vaddr = ph.virtual_addr() as usize;
            let mem_size = ph.mem_size() as usize;
            let data = match ph.get_data(self).unwrap() {
                SegmentData::Undefined(data) => data,
                _ => unreachable!(),
            };
            // 这里在插入一个 MemoryArea 时还需要复制数据
            // 所以我们将 MemorySet 的接口略作修改，最后一个参数为数据源
            memory_set.push(
                vaddr,
                vaddr + mem_size,
                ph.flags().to_attr(), //将elf段的标志转化为我们熟悉的 MemoryAttr
                ByFrame::new(),
                Some((data.as_ptr() as usize, data.len())),
            );
        }
        memory_set
    }
}

trait ToMemoryAttr {
    fn to_attr(&self) -> MemoryAttr;
}

impl ToMemoryAttr for Flags {
    fn to_attr(&self) -> MemoryAttr {
        let mut flags = MemoryAttr::new().set_user();
        if self.is_execute() {
            flags = flags.set_execute();
        }
        flags
    }
}