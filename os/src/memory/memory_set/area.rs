use alloc::boxed::Box;
use crate::memory::paging::{PageTableImpl, PageRange,};
use super::{attr::MemoryAttr, handler::MemoryHandler, };
use crate::consts::PAGE_SIZE;

// 声明中给出所在的虚拟地址区间: [start, end)
// 使用的 MemoryHandler： handler
// 页表项的权限： attr
#[derive(Debug,Clone)]
pub struct MemoryArea {
    start : usize,
    end : usize,
    handler : Box<dyn MemoryHandler>,
    attr : MemoryAttr,
}

impl MemoryArea {
    // 同样是插入、删除映射
    // 遍历虚拟地址区间包含的所有虚拟页，依次利用 handler 完成映射插入/删除
    pub fn map(&self, pt : &mut PageTableImpl) {
        // 使用自己定义的迭代器进行遍历，实现在 src/memory/paging.rs 中
        // 放在下面
        for page in PageRange::new(self.start, self.end) {
            self.handler.map(pt, page, &self.attr);
        }
    }
    fn unmap(&self, pt : &mut PageTableImpl) {
        for page in PageRange::new(self.start, self.end) {
            self.handler.unmap(pt, page);
        }
    }
    // 是否与另一虚拟地址区间相交
    pub fn is_overlap_with(&self, start_addr : usize, end_addr : usize) -> bool {
        let p1 = self.start / PAGE_SIZE;
        let p2 = (self.end - 1) / PAGE_SIZE + 1;
        let p3 = start_addr / PAGE_SIZE;
        let p4 = (end_addr - 1) / PAGE_SIZE + 1;
        !((p1 >= p4) || (p2 <= p3))
    }
    // 初始化
    pub fn new(start_addr : usize, end_addr : usize, handler : Box<dyn MemoryHandler>, attr : MemoryAttr) -> Self {
        MemoryArea{
            start : start_addr,
            end : end_addr,
            handler : handler,
            attr : attr,
        }
    }
    pub fn page_copy(&self, pt: &mut PageTableImpl, src: usize, length: usize) {
        let mut l = length;
        let mut s = src;
        for page in PageRange::new(self.start, self.end) {
            // 交给 MemoryHandler 逐页进行复制
            self.handler.page_copy(
                pt,
                page,
                s,
                if l < PAGE_SIZE { l } else { PAGE_SIZE },
            );
            s += PAGE_SIZE;
            if l >= PAGE_SIZE { l -= PAGE_SIZE; }
        }
    }
}