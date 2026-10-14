//! Slide-show program for bare-metal Raspberry Pi
//!
//! Tested in qemu-system-arm (might not work on real Raspberry Pi!)

#![no_std]
#![no_main]
#![feature(alloc_io)]

extern crate alloc;

use core::{
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering::Relaxed},
};

use alloc::io::Read;

use aarch32_rt::entry;
use arm_pl011_uart::UniqueMmioPointer;
use embedded_graphics::{Drawable, geometry::Point, image::Image};
use semihosting::println;

use pi_demo as _;

static IMAGES: &[(&[u8], &'static str)] = &[
    (
        include_bytes!("../slides/slides.001.bmp.gz"),
        include_str!("../slides/slides.001.txt"),
    ),
    (
        include_bytes!("../slides/slides.002.bmp.gz"),
        include_str!("../slides/slides.002.txt"),
    ),
    (
        include_bytes!("../slides/slides.003.bmp.gz"),
        include_str!("../slides/slides.003.txt"),
    ),
    (
        include_bytes!("../slides/slides.004.bmp.gz"),
        include_str!("../slides/slides.004.txt"),
    ),
    (
        include_bytes!("../slides/slides.005.bmp.gz"),
        include_str!("../slides/slides.005.txt"),
    ),
    (
        include_bytes!("../slides/slides.006.bmp.gz"),
        include_str!("../slides/slides.006.txt"),
    ),
    (
        include_bytes!("../slides/slides.007.bmp.gz"),
        include_str!("../slides/slides.007.txt"),
    ),
    (
        include_bytes!("../slides/slides.008.bmp.gz"),
        include_str!("../slides/slides.008.txt"),
    ),
    (
        include_bytes!("../slides/slides.009.bmp.gz"),
        include_str!("../slides/slides.009.txt"),
    ),
    (
        include_bytes!("../slides/slides.010.bmp.gz"),
        include_str!("../slides/slides.010.txt"),
    ),
    (
        include_bytes!("../slides/slides.011.bmp.gz"),
        include_str!("../slides/slides.011.txt"),
    ),
    (
        include_bytes!("../slides/slides.012.bmp.gz"),
        include_str!("../slides/slides.012.txt"),
    ),
    (
        include_bytes!("../slides/slides.013.bmp.gz"),
        include_str!("../slides/slides.013.txt"),
    ),
    (
        include_bytes!("../slides/slides.014.bmp.gz"),
        include_str!("../slides/slides.014.txt"),
    ),
    (
        include_bytes!("../slides/slides.015.bmp.gz"),
        include_str!("../slides/slides.015.txt"),
    ),
    (
        include_bytes!("../slides/slides.016.bmp.gz"),
        include_str!("../slides/slides.016.txt"),
    ),
    (
        include_bytes!("../slides/slides.017.bmp.gz"),
        include_str!("../slides/slides.017.txt"),
    ),
    (
        include_bytes!("../slides/slides.018.bmp.gz"),
        include_str!("../slides/slides.018.txt"),
    ),
    (
        include_bytes!("../slides/slides.019.bmp.gz"),
        include_str!("../slides/slides.019.txt"),
    ),
    (
        include_bytes!("../slides/slides.020.bmp.gz"),
        include_str!("../slides/slides.020.txt"),
    ),
    (
        include_bytes!("../slides/slides.021.bmp.gz"),
        include_str!("../slides/slides.021.txt"),
    ),
    (
        include_bytes!("../slides/slides.022.bmp.gz"),
        include_str!("../slides/slides.022.txt"),
    ),
];

static IMAGE_NUMBER: AtomicUsize = AtomicUsize::new(0);

/// The entry-point to the Rust application.
///
/// It is called by the start-up.
#[entry]
fn my_main() -> ! {
    let mut board = pi_demo::init();
    let mut vc = pi_demo::videocore::Videocore::new(&mut board.mailbox);
    board
        .uart
        .set_interrupt_masks(arm_pl011_uart::Interrupts::RXI);
    let mut interrupt = unsafe { pi_demo::interrupt::Driver::new() };
    interrupt.enable_irq(pi_demo::interrupt::Interrupt::Uart);
    unsafe {
        aarch32_cpu::interrupt::enable();
    }

    println!("Got videocore version 0x{:08x}", vc.get_firmware_version());
    println!("Got board model 0x{:08x}", vc.get_board_model());
    println!("Got board version 0x{:08x}", vc.get_board_version());
    println!("Got MAC 0x{:012x}", vc.get_mac_address());
    let (base, size) = vc.get_arm_memory();
    println!("Got Arm memory {} MiB @ {:p}", size >> 20, base);
    let (base, size) = vc.get_vc_memory();
    println!("Got VC memory {} MiB @ {:p}", size >> 20, base);
    println!(
        "Got temperature {:0.1} ºC (max {:0.1} ºC)",
        vc.get_temperature(),
        vc.get_max_temperature()
    );

    vc.blank_screen(false);

    let mut fb = vc.create_frambuffer(1920, 1080);

    println!(
        "Screen virt {:?} @ {} bpp",
        vc.get_virt_width_height(),
        vc.get_bpp()
    );
    println!("Screen phys {:?}", vc.get_phys_width_height());

    loop {
        let image_number = IMAGE_NUMBER.load(Relaxed);
        let slide = IMAGES[image_number];
        semihosting::println!(
            "Drawing slide {}/{}\n{}",
            image_number + 1,
            IMAGES.len(),
            slide.1
        );
        let mut decompressor = flate2::read::GzDecoder::new(slide.0);
        let mut image_buffer = alloc::vec::Vec::new();
        let _length = decompressor
            .read_to_end(&mut image_buffer)
            .expect("upacking gzip");

        semihosting::println!("Loading BMP...");
        let bmp = tinybmp::Bmp::from_slice(&image_buffer).expect("invalid bmp");
        semihosting::println!("Creating image...");
        let image = Image::new(&bmp, Point::new(0, 0));

        semihosting::println!("Drawing image...");
        image.draw(&mut fb).expect("drawing");
        semihosting::println!("Done...");

        aarch32_cpu::asm::wfi();
    }
}

#[aarch32_rt::exception(Irq)]
fn interrupt_handler() {
    semihosting::println!("IRQ!");
    let uart_regs =
        unsafe { UniqueMmioPointer::new(NonNull::new(pi_demo::UART_ADDRESS as *mut _).unwrap()) };
    let mut uart = arm_pl011_uart::Uart::new(uart_regs);
    let mut image_number = IMAGE_NUMBER.load(Relaxed);
    if let Ok(Some(ch)) = uart.read_word() {
        semihosting::println!("Reading byte {}", ch as char);
        match ch {
            b',' => {
                image_number = image_number.saturating_sub(1);
                IMAGE_NUMBER.store(image_number, Relaxed);
            }
            b'.' => {
                image_number = image_number.saturating_add(1);
                if image_number >= IMAGES.len() {
                    image_number = IMAGES.len() - 1;
                }
                IMAGE_NUMBER.store(image_number, Relaxed);
            }
            _ => {}
        }
    }
}
