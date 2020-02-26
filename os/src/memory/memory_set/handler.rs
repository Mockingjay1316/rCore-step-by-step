use crate::memory::paging::PageTableImpl;
use super::attr::MemoryAttr;
use crate::memory::alloc_frame; 
use core::fmt::Debug;
use alloc::boxed::Box;

// 定义 MemoryHandler trait
pub trait MemoryHandler: Debug + 'static {
    fn box_clone(&self) -> Box<dyn MemoryHandler>;
    // 需要实现 map, unmap 两函数,不同的接口实现者会有不同的行为
    // 注意 map 并没有 pa 作为参数，因此接口实现者要给出该虚拟页要映射到哪个物理页
    fn map(&self, pt: &mut PageTableImpl, va: usize, attr: &MemoryAttr);
    fn unmap(&self, pt: &mut PageTableImpl, va: usize);
}
impl Clone for Box<dyn MemoryHandler> {
    fn clone(&self) -> Box<dyn MemoryHandler> { self.box_clone() }
}

// 下面给出两种实现 Linear, ByFrame
// 线性映射 Linear: 也就是我们一直在用的带一个偏移量的形式
// 有了偏移量，我们就知道虚拟页要映射到哪个物理页了
#[derive(Debug, Clone)]
pub struct Linear { offset: usize }
impl Linear {
    pub fn new(off: usize) -> Self { Linear { offset: off, }  }
}
impl MemoryHandler for Linear {
    fn box_clone(&self) -> Box<dyn MemoryHandler> { Box::new(self.clone()) }
    fn map(&self, pt: &mut PageTableImpl, va: usize, attr: &MemoryAttr) {
        // 映射到 pa = va - self.offset
        // 同时还使用 attr.apply 修改了原先默认为 R|W|X 的权限
        attr.apply(pt.map(va, va - self.offset));
    }
    fn unmap(&self, pt: &mut PageTableImpl, va: usize) { pt.unmap(va); }
}
// ByFrame: 不知道映射到哪个物理页帧
// 那我们就分配一个新的物理页帧，可以保证不会产生冲突
#[derive(Debug, Clone)]
pub struct ByFrame;
impl ByFrame {
    pub fn new() -> Self { ByFrame {} }
}
impl MemoryHandler for ByFrame {
    fn box_clone(&self) -> Box<dyn MemoryHandler> {
        Box::new(self.clone())
    }
    fn map(&self, pt: &mut PageTableImpl, va: usize, attr: &MemoryAttr) {
        // 分配一个物理页帧作为映射目标
        let frame = alloc_frame().expect("alloc_frame failed!");
        let pa = frame.start_address().as_usize();
        attr.apply(pt.map(va, pa));
    }
    fn unmap(&self, pt: &mut PageTableImpl, va: usize) {
        pt.unmap(va);
    }
}