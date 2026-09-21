#![no_std]

extern crate alloc;

use core::sync::atomic::{AtomicBool, Ordering};

use semihosting::println;

pub mod mailbox;
pub mod videocore;

/// Raspberry Pi 2 and 3 MMIO base address
#[cfg(arm_architecture = "v7-a")]
pub const MMIO_BASE: usize = 0x3F00_0000;

/// Raspberry Pi Zero MMIO base address
#[cfg(not(arm_architecture = "v7-a"))]
pub const MMIO_BASE: usize = 0x2000_0000;

/// Our heap
#[global_allocator]
static HEAP: embedded_alloc::LlffHeap = embedded_alloc::LlffHeap::empty();

pub struct Board {
    pub mailbox: mailbox::Driver,
}

pub fn init() -> Board {
    static HAS_INIT: AtomicBool = AtomicBool::new(false);
    if HAS_INIT.swap(true, Ordering::SeqCst) {
        panic!("Double init!");
    }

    semihosting::println!("{:?}", aarch32_cpu::register::Sctlr::read());

    const HEAP_SIZE: usize = 1024 * 1024 * 32;
    static mut HEAP_SPACE: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

    unsafe {
        HEAP.init(&raw mut HEAP_SPACE as usize, HEAP_SIZE);
    }

    // Safety: Init only called once
    let mailbox = unsafe { mailbox::Driver::new() };

    Board { mailbox }
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("PANIC: {}", info);
    semihosting::process::exit(1);
}
