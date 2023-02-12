#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(map_try_insert)]
#![feature(vec_into_raw_parts)]
#![allow(unused)]
#![feature(core_intrinsics)]
#![feature(error_in_core)]
#![feature(associated_type_bounds)]
#![feature(trait_upcasting)]
extern crate alloc;
extern crate rv_plic;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate log;

use crate::{config::CPU_NUM, mm::init_kernel_space, sbi::send_ipi};
use core::arch::{asm, global_asm};

#[macro_use]
mod console;
mod config;
#[macro_use]
mod fs;
mod lang_items;
mod loader;
mod logger;
mod mm;
mod plic;
mod sbi;
mod syscall;
mod task;
mod sync;
mod timer;
mod trap;
#[macro_use]
mod uart;
mod trace;
mod lkm;
mod drivers;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| unsafe { (a as *mut u8).write_volatile(0) });
}

#[no_mangle]
pub fn rust_main(hart_id: usize) -> ! {
    if hart_id == 0 {
        clear_bss();
        logger::init();
        mm::init();
        debug!("[kernel {}] Hello, world!", hart_id);
        mm::remap_test();
        trace::init();
        trace::trace_test();
        trap::init();
        plic::init();
        plic::init_hart(hart_id);
        uart::init();
        lkm::init();

        extern "C" {
            fn boot_stack();
            fn boot_stack_top();
        }

        debug!(
            "boot_stack {:#x} top {:#x}",
            boot_stack as usize, boot_stack_top as usize
        );
        debug!("trying to add initproc");
        task::add_initproc();
        debug!("initproc added to task manager!");

        unsafe {
            let satp: usize;
            let sp: usize;
            asm!("csrr {}, satp", out(reg) satp);
            asm!("mv {}, sp", out(reg) sp);
            println_hart!("satp: {:#x}, sp: {:#x}", hart_id, satp, sp);
        }

        for i in 1..CPU_NUM {
            debug!("[kernel {}] Start {}", hart_id, i);
            let mask: usize = 1 << i;
            send_ipi(&mask as *const _ as usize);
        }
    } else {
        let hart_id = task::hart_id();

        init_kernel_space();

        unsafe {
            let satp: usize;
            let sp: usize;
            asm!("csrr {}, satp", out(reg) satp);
            asm!("mv {}, sp", out(reg) sp);
            println_hart!("satp: {:#x}, sp: {:#x}", hart_id, satp, sp);
        }
        trap::init();
        plic::init_hart(hart_id);
    }

    println_hart!("Hello", hart_id);

    if hart_id == 0 {
        fs::list_apps();
        // TODO 为什么在内核态可以正常读取内容，但是从用户态转到内核态后无法进行读取
        // fs::read_all_file();
    }
    timer::set_next_trigger();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
