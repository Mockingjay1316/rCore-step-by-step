use spin::Mutex;
use alloc::collections::VecDeque;
use crate::process::{ Tid, current_tid, yield_now, wake_up };

#[derive(Default)]
pub struct Condvar {
    // 加了互斥锁的 Tid 队列
    // 存放等待此条件变量的众多线程
    wait_queue: Mutex<VecDeque<Tid>>,
}

impl Condvar {
    pub fn new() -> Self {
        Condvar::default()
    }

    // 当前线程等待某种条件满足才能继续执行
    pub fn wait(&self) {
        // 将当前 Tid 加入此条件变量的等待队列
        self.wait_queue
            .lock()
            .push_back(current_tid());
        // 当前线程放弃 CPU 资源
        yield_now();
    }

    // 条件满足
    pub fn notify(&self) {
        // 弹出等待队列中的一个线程
        let tid = self.wait_queue.lock().pop_front();
        if let Some(tid) = tid {
            // 唤醒该线程
            wake_up(tid);
        }
    }
}