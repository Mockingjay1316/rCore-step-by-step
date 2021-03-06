use core::cell::UnsafeCell;
use alloc::boxed::Box;
use crate::process::Tid;
use crate::process::structs::*;
use crate::process::thread_pool::ThreadPool;
use crate::interrupt::*;

// 调度单元 Processor 的内容
pub struct ProcessorInner {
    // 线程池
    pool: Box<ThreadPool>,
    // idle 线程
    idle: Box<Thread>,
    // 当前正在运行的线程
    current: Option<(Tid, Box<Thread>)>,
}

pub struct Processor {
    inner: UnsafeCell<Option<ProcessorInner>>,
}
unsafe impl Sync for Processor {}

impl Processor {
    // 新建一个空的 Processor
    pub const fn new() -> Processor {
        Processor {  inner: UnsafeCell::new(None),  }
    }
    // 传入 idle 线程，以及线程池进行初始化
    pub fn init(&self, idle: Box<Thread>, pool: Box<ThreadPool>) {
        unsafe {
            *self.inner.get() = Some(
                ProcessorInner {
                    pool,
                    idle,
                    current: None,
                }
            );
        }
    }
    // 内部可变性：获取包裹的值的可变引用
    pub fn inner(&self) -> &mut ProcessorInner {
        unsafe { &mut *self.inner.get() }
            .as_mut()
            .expect("Processor is not initialized!")
    }
    // 通过线程池新增线程
    pub fn add_thread(&self, thread: Box<Thread>) {
        self.inner().pool.add(thread);
    }

    pub fn idle_main(&self) -> ! {
        let inner = self.inner();
        // 在 idle 线程刚进来时禁用异步中断
        disable_and_store();

        loop {
            // 如果从线程池中获取到一个可运行线程
            if let Some(thread) = inner.pool.acquire() {
                // 将自身的正在运行线程设置为刚刚获取到的线程
                inner.current = Some(thread);
                // 从正在运行的线程 idle 切换到刚刚获取到的线程
                //println!("\n>>>> will switch_to thread {} in idle_main!", inner.current.as_mut().unwrap().0);
                inner.idle.switch_to(
                    &mut *inner.current.as_mut().unwrap().1
                );

                // 上个线程时间耗尽，切换回调度线程 idle
                //println!("<<<< switch_back to idle in idle_main!");
                // 此时 current 还保存着上个线程
                let (tid, thread) = inner.current.take().unwrap();
                // 通知线程池这个线程需要将资源交还出去
                inner.pool.retrieve(tid, thread);
            }
            // 如果现在并无任何可运行线程
            else {
                // 打开异步中断，并等待异步中断的到来
                enable_and_wfi();
                // 异步中断处理返回后，关闭异步中断
                disable_and_store();
            }
        }
    }

    pub fn tick(&self) {
        let inner = self.inner();
        if !inner.current.is_none() {
            // 如果当前有在运行线程
            if inner.pool.tick() {
                // 如果返回true, 表示当前运行线程时间耗尽，需要被调度出去

                // 我们要进入 idle 线程了，因此必须关闭异步中断
                // 我们可没保证 switch_to 前后 sstatus 寄存器不变
                // 因此必须手动保存
                let flags = disable_and_store();

                // 切换到 idle 线程进行调度
                inner.current
                    .as_mut()
                    .unwrap()
                    .1
                    .switch_to(&mut inner.idle);

                // 之后某个时候又从 idle 线程切换回来
                // 恢复 sstatus 寄存器继续中断处理
                restore(flags);
            }
        }
    }

    pub fn run(&self) {
        // 运行，也就是从启动线程切换到调度线程 idle
        Thread::get_boot_thread().switch_to(&mut self.inner().idle);
    }

    pub fn exit(&self, code: usize) -> ! {
        // 由于要切换到 idle 线程，必须先关闭时钟中断
        disable_and_store();
        // 由于自己正在执行，可以通过这种方式获取自身的 tid
        let inner = self.inner();
        let tid = inner.current.as_ref().unwrap().0;

        // 通知线程池这个线程退出啦！
        inner.pool.exit(tid);
        println!("thread {} exited, exit code = {}", tid, code);

        // 加入这个判断
        // 如果有一个线程正在等待当前线程运行结束
        // 将其唤醒
        if let Some(wait) = inner.current.as_ref().unwrap().1.wait {
            inner.pool.wakeup(wait);
        }

        // 切换到 idle 线程决定下一个运行哪个线程
        inner.current
            .as_mut()
            .unwrap()
            .1
            .switch_to(&mut inner.idle);

        loop {}
    }

    pub fn yield_now(&self) {
        let inner = self.inner();
        if !inner.current.is_none() {
            unsafe {
                // 由于要进入 idle 线程，必须关闭异步中断
                // 手动保存之前的 sstatus
                let flags = disable_and_store();
                let tid = inner.current.as_mut().unwrap().0;
                let thread_info = inner.pool.threads[tid].as_mut().expect("thread not existed when yielding");
                // 修改线程状态
                thread_info.status = Status::Sleeping;
                // 切换到 idle 线程
                inner.current
                    .as_mut()
                    .unwrap()
                    .1
                    .switch_to(&mut *inner.idle);

                // 从 idle 线程切换回来
                // 恢复 sstatus
                restore(flags);
            }
        }
    }

    pub fn wake_up(&self, tid: Tid) {
        let inner = self.inner();
        inner.pool.wakeup(tid);
    }

    pub fn current_tid(&self) -> usize {
        self.inner().current.as_mut().unwrap().0 as usize
    }
}