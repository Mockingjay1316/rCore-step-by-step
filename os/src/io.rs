use crate::sbi;
use core::fmt::{ self, Write };

struct Stdout;

pub fn getchar() -> char {
    let c = sbi::console_getchar() as u8;

    match c {
        255 => '\0',
        c => c as char
    }
}
// 调用 OpenSBI 接口
pub fn getchar_option() -> Option<char> {
    let c = sbi::console_getchar() as isize;
    match c {
        -1 => None,
        c => Some(c as u8 as char)
    }
}

// 输出一个字符
pub fn putchar(ch: char) {
    sbi::console_putchar(ch as u8 as usize);
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