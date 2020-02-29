use crate::syscall::sys_write;
use crate::syscall::sys_read;
use core::fmt::{ self, Write };

// 每个进程默认打开三个文件
// 标准输入 stdin fd = 0
// 标准输出 stdout fd = 1
// 标准错误输出 stderr fd = 2
pub const STDIN: usize = 0;
struct Stdout;

// 调用 sys_read 从标准输入读入一个字符
pub fn getc() -> u8 {
    let mut c = 0u8;
    assert_eq!(sys_read(STDIN, &mut c, 1), 1);
    c
}

// 输出一个字符
pub fn putchar(ch: char) {
    sys_write(ch as u8);
}

// 输出一个字符串
pub fn puts(s: &str) {
    for ch in s.chars() {
        putchar(ch);
    }
}

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        puts(s);
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        $crate::io::_print(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}