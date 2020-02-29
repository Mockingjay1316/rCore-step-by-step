use crate::process::scheduler::Scheduler;
use crate::process::structs::*;
use crate::alloc::{
    vec::Vec,
    boxed::Box,
};
use crate::process::Tid;

// 线程池每个位置的信息
pub struct ThreadInfo {
    // 占据这个位置的线程当前运行状态
    pub status: Status,
    // 占据这个位置的线程
    pub thread: Option<Box<Thread>>,
}

pub struct ThreadPool {
    // 线程池
    // 如果一个位置是 None 表示未被线程占据
    pub threads: Vec<Option<ThreadInfo>>,
    // 调度算法
    // 这里的 dyn Scheduler 是 Trait object 语法
    // 表明 Box 里面的类型实现了 Scheduler Trait
    scheduler: Box<dyn Scheduler>,
}

impl ThreadPool {
    // 新建一个线程池，其最大可容纳 size 个线程，使用调度器 scheduler
    pub fn new(size: usize, scheduler: Box<dyn Scheduler>) -> ThreadPool {
        ThreadPool {
            threads: {
                let mut v = Vec::new();
                v.resize_with(size, Default::default);
                v
            },
            scheduler,
        }
    }
    // 在线程池中找一个编号最小的空着的位置
    // 将编号作为 Tid 返回
    fn alloc_tid(&self) -> Tid {
        for (i, info) in self.threads.iter().enumerate() {
            if info.is_none() {
                return i;
            }
        }
        panic!("alloc tid failed!");
    }

    // 加入一个可立即开始运行的线程
    // 线程状态 Uninitialized -> Ready
    pub fn add(&mut self, _thread: Box<Thread>) {
        // 分配 Tid
        let tid = self.alloc_tid();
        // 修改线程池对应位置的信息
        self.threads[tid] = Some(
            ThreadInfo {
                // 状态：随时准备运行，等待 CPU 资源中
                status: Status::Ready,
                // 传入线程
                thread: Some(_thread),
            }
        );
        // 将线程的 Tid 加入调度器
        // 提醒调度器给这个线程分配 CPU 资源
        self.scheduler.push(tid);
    }

    // 从线程池中取一个线程开始运行
    // 线程状态 Ready -> Running
    pub fn acquire(&mut self) -> Option<(Tid, Box<Thread>)> {
        // 调用 Scheduler::pop ，从调度算法中获取接下来要运行的 Tid
        if let Some(tid) = self.scheduler.pop() {
            // 获取并更新线程池对应位置的信息
            let mut thread_info = self.threads[tid].as_mut().expect("thread not exist!");
            // 将线程状态改为 Running
            thread_info.status = Status::Running(tid);
            return Some((tid, thread_info.thread.take().expect("thread not exist!")));
        }
        else {
            return None;
        }
    }
    // 这个线程已运行了太长时间或者已运行结束，需要交出CPU资源
    // 但是要提醒线程池它仍需要分配 CPU 资源
    pub fn retrieve(&mut self, tid: Tid, thread: Box<Thread>) {
        // 线程池位置为空，表明这个线程刚刚通过 exit 退出
        if self.threads[tid].is_none() {
            // 不需要 CPU 资源了，退出
            return;
        }
        // 获取并修改线程池对应位置的信息
        let mut thread_info = self.threads[tid].as_mut().expect("thread not exist!");
        thread_info.thread = Some(thread);
        // 此时状态可能是 Status::Sleeping(线程可能会自动放弃 CPU 资源，进入睡眠状态),
        // 直到被唤醒之前都不必给它分配。
        // 而如果此时状态是Running,就说明只是单纯的耗尽了这次分配CPU资源,但还要占用CPU资源继续执行。
        if let Status::Running(_) = thread_info.status {
            // Running -> Ready
            thread_info.status = Status::Ready;
            // 通知线程池继续给此线程分配资源
            self.scheduler.push(tid);
        }
    }
    // Scheduler 的简单包装：时钟中断时查看当前所运行线程是否要切换出去
    pub fn tick(&mut self) -> bool {
        let ret = self.scheduler.tick();
        ret
    }
    // 这个线程已经退出了，线程状态 Running -> Exited
    pub fn exit(&mut self, tid: Tid) {
        // 清空线程池对应位置
        self.threads[tid] = None;
        // 通知调度器
        self.scheduler.exit(tid);
    }
    pub fn wakeup(&mut self, tid: Tid) {
        let proc = self.threads[tid].as_mut().expect("thread not exist when waking up");
        proc.status = Status::Ready;
        self.scheduler.push(tid);
    }
}