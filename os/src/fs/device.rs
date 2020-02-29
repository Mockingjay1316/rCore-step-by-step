use spin::RwLock;
use rcore_fs::dev::*;

pub struct MemBuf(RwLock<&'static mut [u8]>); //一块用于模拟磁盘的内存

impl MemBuf {
    // 初始化参数为磁盘的头尾虚拟地址
    pub unsafe fn new(begin: usize, end: usize) -> Self {
        use core::slice;
        MemBuf(
            // 我们使用读写锁
            // 可以有多个线程同时获取 & 读
            // 但是一旦有线程获取 &mut 写，那么其他所有线程都将被阻塞
            RwLock::new(
                slice::from_raw_parts_mut(begin as *mut u8, end - begin)
            )
        )
    }
}
// 作为文件系统所用的设备驱动，只需实现下面三个接口
// 而在设备实际上是内存的情况下，实现变的极其简单
impl Device for MemBuf {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        let slice = self.0.read();
        let len = buf.len().min(slice.len() - offset);
        buf[..len].copy_from_slice(&slice[offset..offset + len]);
        Ok(len)
    }
    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
        let mut slice = self.0.write();
        let len = buf.len().min(slice.len() - offset);
        slice[offset..offset + len].copy_from_slice(&buf[..len]);
        Ok(len)
    }
    fn sync(&self) -> Result<()> {
        Ok(())
    }
}