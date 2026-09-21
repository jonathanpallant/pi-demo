//! Semihosting hello-world.

#![no_std]
#![no_main]
#![feature(alloc_io)]

extern crate alloc;

use alloc::io::Read;

use aarch32_rt::entry;
use embedded_graphics::{Drawable, geometry::Point, image::Image};
use semihosting::println;

use pi_demo as _;

static IMAGE_GZ: &'static [u8] = include_bytes!("test_card.bmp.gz");

/// The entry-point to the Rust application.
///
/// It is called by the start-up.
#[entry]
fn my_main() -> ! {
    let mut board = pi_demo::init();
    let x = 1.0f64;
    let y = x * 2.0;
    println!("Hello, this is semihosting! x = {:0.3}, y = {:0.3}", x, y);

    let mut vc = pi_demo::videocore::Videocore::new(&mut board.mailbox);

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

    let mut decompressor = flate2::read::GzDecoder::new(IMAGE_GZ);
    let mut image_buffer = alloc::vec::Vec::new();
    let _length = decompressor
        .read_to_end(&mut image_buffer)
        .expect("upacking gzip");

    let bmp = tinybmp::Bmp::from_slice(&image_buffer).expect("invalid bmp");
    let image = Image::new(&bmp, Point::new(0, 0));

    image.draw(&mut fb).expect("drawing");

    loop {}
}
