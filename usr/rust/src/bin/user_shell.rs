#![no_std]
#![no_main]
#![feature(alloc)]

extern crate alloc;

#[macro_use]
extern crate user;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;

use user::io::getc;
use user::syscall::sys_exec;
use alloc::string::String;

#[no_mangle]
pub fn main() {
   println!("Rust user shell");
   // 保存本行已经输入的内容
   let mut line: String = String::new();
   print!(">> ");
   loop {
       let c = getc();
       match c {
           LF | CR => {
               // 如果遇到回车或换行
               println!("");
               if !line.is_empty() {
                   line.push('\0');
                   println!("searching for program {}", line);
                   // 使用系统调用执行程序
                   sys_exec(line.as_ptr());
                   // 清空本行内容
                   line.clear();
               }
               print!(">> ");
           },
           _ => {
               // 否则正常输入
               print!("{}", c as char);
               line.push(c as char);
           }
       }
   }
}