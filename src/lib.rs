#![no_std]

extern crate alloc;

use core::{
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
};

use arm_pl011_uart::UniqueMmioPointer;

pub mod mailbox;
pub mod videocore;

/// Raspberry Pi 2 and 3 MMIO base address
#[cfg(arm_architecture = "v7-a")]
pub const MMIO_BASE: usize = 0x3F00_0000;

/// Raspberry Pi Zero MMIO base address
#[cfg(not(arm_architecture = "v7-a"))]
pub const MMIO_BASE: usize = 0x2000_0000;

pub const UART_ADDRESS: usize = MMIO_BASE + 0x0020_1000;

/// Our heap
#[global_allocator]
static HEAP: embedded_alloc::LlffHeap = embedded_alloc::LlffHeap::empty();

pub struct Board {
    pub mailbox: mailbox::Driver,
    pub uart: arm_pl011_uart::Uart<'static>,
}

pub fn init() -> Board {
    static HAS_INIT: AtomicBool = AtomicBool::new(false);
    if HAS_INIT.swap(true, Ordering::SeqCst) {
        panic!("Double init!");
    }

    const HEAP_SIZE: usize = 1024 * 1024 * 32;
    static mut HEAP_SPACE: [u8; HEAP_SIZE] = [0u8; HEAP_SIZE];

    unsafe {
        HEAP.init(&raw mut HEAP_SPACE as usize, HEAP_SIZE);
    }

    // Safety: Init only called once
    let mailbox = unsafe { mailbox::Driver::new() };

    let uart_regs =
        unsafe { UniqueMmioPointer::new(NonNull::new(UART_ADDRESS as *mut _).unwrap()) };
    let mut uart = arm_pl011_uart::Uart::new(uart_regs);
    let line_config = arm_pl011_uart::LineConfig {
        data_bits: arm_pl011_uart::DataBits::Bits8,
        parity: arm_pl011_uart::Parity::None,
        stop_bits: arm_pl011_uart::StopBits::One,
    };
    uart.enable(line_config, 115_200, 250_000_000)
        .expect("uart_enable");

    Board { mailbox, uart }
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    semihosting::println!("PANIC: {}", info);
    semihosting::process::exit(1);
}
