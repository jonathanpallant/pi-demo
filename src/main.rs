//! Slide-show program for bare-metal Raspberry Pi
//!
//! Tested in qemu-system-arm (might not work on real Raspberry Pi!)

#![no_std]
#![no_main]
#![feature(alloc_io)]

extern crate alloc;

use alloc::io::Read;

use aarch32_rt::entry;
use embedded_graphics::{Drawable, geometry::Point, image::Image};
use semihosting::println;

use pi_demo as _;

static IMAGES: &[&[u8]] = &[
    include_bytes!("../slides/slides.001.bmp.gz"),
    include_bytes!("../slides/slides.002.bmp.gz"),
    include_bytes!("../slides/slides.003.bmp.gz"),
    include_bytes!("../slides/slides.004.bmp.gz"),
    include_bytes!("../slides/slides.005.bmp.gz"),
    include_bytes!("../slides/slides.006.bmp.gz"),
    include_bytes!("../slides/slides.007.bmp.gz"),
    include_bytes!("../slides/slides.008.bmp.gz"),
    include_bytes!("../slides/slides.009.bmp.gz"),
    include_bytes!("../slides/slides.010.bmp.gz"),
    include_bytes!("../slides/slides.011.bmp.gz"),
    include_bytes!("../slides/slides.012.bmp.gz"),
    include_bytes!("../slides/slides.013.bmp.gz"),
    include_bytes!("../slides/slides.014.bmp.gz"),
    include_bytes!("../slides/slides.015.bmp.gz"),
    include_bytes!("../slides/slides.016.bmp.gz"),
    include_bytes!("../slides/slides.017.bmp.gz"),
    include_bytes!("../slides/slides.018.bmp.gz"),
    include_bytes!("../slides/slides.019.bmp.gz"),
    include_bytes!("../slides/slides.020.bmp.gz"),
    include_bytes!("../slides/slides.021.bmp.gz"),
    include_bytes!("../slides/slides.022.bmp.gz"),
    include_bytes!("../slides/slides.023.bmp.gz"),
];

/// The entry-point to the Rust application.
///
/// It is called by the start-up.
#[entry]
fn my_main() -> ! {
    let mut board = pi_demo::init();
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

    let mut image_number = 0;
    loop {
        semihosting::println!("Drawing slide {}/{}", image_number + 1, IMAGES.len());
        let image_contents = IMAGES[image_number];
        let mut decompressor = flate2::read::GzDecoder::new(image_contents);
        let mut image_buffer = alloc::vec::Vec::new();
        let _length = decompressor
            .read_to_end(&mut image_buffer)
            .expect("upacking gzip");

        let bmp = tinybmp::Bmp::from_slice(&image_buffer).expect("invalid bmp");
        let image = Image::new(&bmp, Point::new(0, 0));

        image.draw(&mut fb).expect("drawing");

        loop {
            if let Ok(Some(ch)) = board.uart.read_word() {
                semihosting::println!("Reading byte {}", ch as char);
                match ch {
                    b',' => {
                        image_number = image_number.saturating_sub(1);
                        break;
                    }
                    b'.' => {
                        image_number = image_number.saturating_add(1);
                        if image_number >= IMAGES.len() {
                            image_number = IMAGES.len() - 1;
                        }
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}
