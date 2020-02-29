use alloc::{ collections::VecDeque, sync::Arc };
use spin::Mutex;
use crate::process;
use crate::sync::condvar::*;
use lazy_static::*;

pub struct Stdin {
    // 字符队列
    buf: Mutex<VecDeque<char>>,
    // 条件变量
    pushed: Condvar,
}

impl Stdin {
    pub fn new() -> Self {
        Stdin {
            buf: Mutex::new(VecDeque::new()),
            pushed: Condvar::new(),
        }
    }

    // 生产者：输入字符
    pub fn push(&self, ch: char) {
        // 将字符加入字符队列
        self.buf
            .lock()
            .push_back(ch);
        // 如果此时有线程正在等待队列非空才能继续下去
        // 将其唤醒
        self.pushed.notify();
    }

    // 消费者：取出字符
    // 运行在请求字符输入的线程上
    pub fn pop(&self) -> char {
        loop {
            // 将代码放在 loop 里面防止再复制一遍

            // 尝试获取队首字符
            let ret = self.buf.lock().pop_front();
            match ret {
                Some(ch) => {
                    // 获取到了直接返回
                    return ch;
                },
                None => {
                    // 否则队列为空，通过 getc -> sys_read 获取字符的当前线程放弃 CPU 资源
                    // 进入阻塞状态等待唤醒
                    self.pushed.wait();

                    // 被唤醒后回到循环开头，此时可直接返回
                }
            }
        }
    }
}

lazy_static! {
    pub static ref STDIN: Arc<Stdin> = Arc::new(Stdin::new());
}