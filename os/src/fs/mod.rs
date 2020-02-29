mod device;

use lazy_static::*;
use rcore_fs::vfs::*;
use rcore_fs_sfs::SimpleFileSystem;
use alloc::{ sync::Arc, vec::Vec };

lazy_static! {
    pub static ref ROOT_INODE: Arc<dyn INode> = {
        // 创建内存模拟的"磁盘"设备
        let device = {
            extern "C" {
                fn _user_img_start();
                fn _user_img_end();
            };
            let start = _user_img_start as usize;
            let end = _user_img_end as usize;
            Arc::new(unsafe { device::MemBuf::new(start, end) })
        };
        // 由于我们在打包磁盘文件时就使用 SimpleFileSystem
        // 所以我们必须使用简单文件系统 SimpleFileSystem 打开该设备进行初始化
        let sfs = SimpleFileSystem::open(device).expect("failed to open SFS");
        // 返回该文件系统的根 INode
        sfs.root_inode()
    };
}

pub trait INodeExt {
    fn read_as_vec(&self) -> Result<Vec<u8>>;
}

impl INodeExt for dyn INode {
    // 将这个 INode 对应的文件读取到一个数组中
    fn read_as_vec(&self) -> Result<Vec<u8>> {
        let size = self.metadata()?.size;
        let mut buf = Vec::with_capacity(size);
        unsafe { buf.set_len(size); }
        self.read_at(0, buf.as_mut_slice())?;
        Ok(buf)
    }
}

pub fn init() {
    println!("available programs in rust/ are:");
    let mut id = 0;
    // 查找 rust 文件夹并返回其对应的 INode
    let mut rust_dir = ROOT_INODE.lookup("rust").unwrap();
    // 遍历里面的文件并输出
    // 实际上打印了所有 rust 目录下的用户程序
    while let Ok(name) = rust_dir.get_entry(id) {
        id += 1;
        println!("  {}", name);
    }
    println!("++++ setup fs! ++++")
}