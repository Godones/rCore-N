#![no_std]
#![no_main]
use syscall::getpid;
use user_lib::println;

#[no_mangle]
pub fn main() -> i32 {
    println!("[hello world] from pid: {}", getpid());
    0
}