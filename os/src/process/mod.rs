pub mod structs;
pub mod scheduler;
pub mod thread_pool;
pub mod processor;

use structs::Thread;
use processor::Processor;
use scheduler::RRScheduler;
use thread_pool::ThreadPool;
use alloc::boxed::Box;
static CPU: Processor = Processor::new();

use crate::fs::{
    ROOT_INODE,
    INodeExt
};

pub type Tid = usize;
pub type ExitCode = usize;

#[no_mangle]
pub extern "C" fn hello_thread(arg: usize) -> ! {
    println!("begin of thread {}", arg);
    for i in 0..800 {
        print!("{}", arg);
    }
    println!("\nend  of thread {}", arg);
    // 通知 CPU 自身已经退出
    exit(0);
    loop {}
}

pub fn init() {
    // 使用 Round Robin Scheduler
    let scheduler = RRScheduler::new(1);
    // 新建线程池
    let thread_pool = ThreadPool::new(100, Box::new(scheduler));
    // 新建内核线程 idle ，其入口为 Processor::idle_main
    let idle = Thread::new_kernel(Processor::idle_main as usize);
    // 我们需要传入 CPU 的地址作为参数
    idle.append_initial_arguments([&CPU as *const Processor as usize, 0, 0]);
    // 初始化 CPU
    CPU.init(idle, Box::new(thread_pool));

    // 依次新建 5 个内核线程并加入调度单元
    for i in 0..5 {
        CPU.add_thread({
            let thread = Thread::new_kernel(hello_thread as usize);
            // 传入一个编号作为参数
            thread.append_initial_arguments([i, 0, 0]);
            thread
        });
    }
    println!("Initialized kernel thread!");

    let data = ROOT_INODE
        .lookup("rust/hello_world")
        .unwrap()
        .read_as_vec()
        .unwrap();
    let user_thread = unsafe { Thread::new_user(data.as_slice()) };
    CPU.add_thread(user_thread);
    println!("++++ setup process!   ++++");
}

pub fn tick() {
    CPU.tick();
}

pub fn run() {
    CPU.run();
}

pub fn exit(code: usize) {
    CPU.exit(code);
}