use crate::context::TrapFrame;
use crate::process;

pub const SYS_READ: usize = 63;
pub const SYS_WRITE: usize = 64;
pub const SYS_EXIT: usize = 93;
pub const SYS_EXEC: usize = 221;

pub fn syscall(id: usize, args: [usize; 3], tf: &mut TrapFrame) -> isize {
    match id {
        SYS_READ => {
            sys_read(args[0], args[1] as *mut u8, args[2])
        },
        SYS_WRITE => {
            print!("{}", args[0] as u8 as char);
            0
        },
        SYS_EXIT => {
            sys_exit(args[0]);
            0
        },
        SYS_EXEC => {
            sys_exec(args[0] as *const u8)
        },
        _ => {
            panic!("unknown syscall id {}", id);
        },
    }
}

fn sys_read(fd: usize, base: *mut u8, len: usize) -> isize {
    unsafe {
        *base = crate::fs::stdio::STDIN.pop() as u8;
    }
    return 1;
}

fn sys_exit(code: usize) {
    process::exit(code);
}

pub unsafe fn from_cstr(s: *const u8) -> &'static str {
    use core::{ slice, str };
    // 使用迭代器获得字符串长度
    let len = (0usize..).find(|&i| *s.add(i) == 0).unwrap();
    str::from_utf8(slice::from_raw_parts(s, len)).unwrap()
}

fn sys_exec(path: *const u8) -> isize {
    let valid = process::execute(unsafe { from_cstr(path) }, Some(process::current_tid()));
    // 如果正常执行，则阻塞终端线程，等到启动的这个用户线程运行结束
    if valid { process::yield_now(); }
    // 不能正常执行，直接返回；或者被启动线程结束后唤醒终端线程之后返回
    return 0;
}