mod frame_allocator;
use frame_allocator::SEGMENT_TREE_ALLOCATOR as FRAME_ALLOCATOR;
use riscv::addr::{
    // 分别为虚拟地址、物理地址、虚拟页、物理页帧
    // 非常方便，之后会经常用到
    // 用法可参见 https://github.com/rcore-os/riscv/blob/master/src/addr.rs
    VirtAddr,
    PhysAddr,
    Page,
    Frame
};

pub fn init(l: usize, r: usize) {
    FRAME_ALLOCATOR.lock().init(l, r);
    println!("++++ setup memory!    ++++");
}
pub fn alloc_frame() -> Option<Frame> {
    //将物理页号转为物理页帧
    Some(Frame::of_ppn(FRAME_ALLOCATOR.lock().alloc()))
}
pub fn dealloc_frame(f: Frame) {
    FRAME_ALLOCATOR.lock().dealloc(f.number())
}