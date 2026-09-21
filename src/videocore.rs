//! Talk to the video core

use arbitrary_int::u28;

use crate::mailbox::{Channel, Driver as Mailbox, Message};

pub struct FrameBuffer32 {
    pub base_ptr: *mut u32,
    width: u32,
    height: u32,
    stride: usize,
}

impl FrameBuffer32 {
    pub fn set_pixel(&mut self, pos: (u32, u32), pixel: u32) {
        let (x, y) = pos;
        if x < self.width && y < self.height {
            let byte_offset = (self.stride * (y as usize)) + (x as usize * 4);
            let ptr = unsafe { self.base_ptr.byte_add(byte_offset) };
            // semihosting::println!("{} x {} (@ {:010p}) = 0x{:08x}", x, y, ptr, pixel);
            unsafe {
                ptr.write_volatile(pixel);
            }
        } else {
            panic!("Off screen draw at {:?}", pos);
        }
    }
}

pub struct Videocore<'a> {
    mailbox: &'a mut Mailbox,
}

impl<'a> Videocore<'a> {
    pub fn new(mailbox: &'a mut Mailbox) -> Self {
        Self { mailbox }
    }

    pub fn get_firmware_version(&mut self) -> u32 {
        self.get_u32(0x00000001, 0)
    }

    pub fn get_board_model(&mut self) -> u32 {
        self.get_u32(0x00010001, 0)
    }

    pub fn get_board_version(&mut self) -> u32 {
        self.get_u32(0x00010002, 0)
    }

    pub fn get_mac_address(&mut self) -> u64 {
        self.get_u64(0x00010003)
    }

    pub fn get_board_serial(&mut self) -> u64 {
        self.get_u64(0x00010004)
    }

    pub fn get_arm_memory(&mut self) -> (*const u8, usize) {
        let (ptr, size) = self.get_u32_pair(0x00010005, (0, 0));
        let ptr = ptr as *const u8;
        let size = size as usize;
        (ptr, size)
    }

    pub fn get_vc_memory(&mut self) -> (*const u8, usize) {
        let (ptr, size) = self.get_u32_pair(0x00010006, (0, 0));
        let ptr = ptr as *const u8;
        let size = size as usize;
        (ptr, size)
    }

    pub fn get_temperature(&mut self) -> f32 {
        let (_, thou_degree) = self.get_u32_pair(0x00030006, (0, 0));
        (thou_degree as f32) / 1000.0
    }

    pub fn get_max_temperature(&mut self) -> f32 {
        let (_, thou_degree) = self.get_u32_pair(0x0003000a, (0, 0));
        (thou_degree as f32) / 1000.0
    }

    pub fn get_virt_width_height(&mut self) -> (u32, u32) {
        let (width, height) = self.get_u32_pair(0x00040004, (0, 0));
        (width, height)
    }

    pub fn set_virt_width_height(&mut self, width: u32, height: u32) -> (u32, u32) {
        let (width, height) = self.get_u32_pair(0x00048004, (width, height));
        (width, height)
    }

    pub fn get_phys_width_height(&mut self) -> (u32, u32) {
        let (width, height) = self.get_u32_pair(0x00040003, (0, 0));
        (width, height)
    }

    pub fn set_phys_width_height(&mut self, width: u32, height: u32) -> (u32, u32) {
        let (width, height) = self.get_u32_pair(0x00048003, (width, height));
        (width, height)
    }

    pub fn get_bpp(&mut self) -> u32 {
        let bpp = self.get_u32(0x00040005, 0);
        bpp
    }

    pub fn set_bpp(&mut self, bpp: u32) -> u32 {
        let bpp = self.get_u32(0x00048005, bpp);
        bpp
    }

    pub fn blank_screen(&mut self, blank: bool) {
        let _ = self.get_u32(0x00040002, if blank { 1 } else { 0 });
    }

    pub fn allocate_fb(&mut self) -> (*mut u32, usize) {
        // ask for four byte alignment
        let (ptr, size) = self.get_u32_pair(0x00040001, (4, 0));
        let ptr = ptr as *mut u32;
        let size = size as usize;
        (ptr, size)
    }

    pub fn create_frambuffer(&mut self, width: u32, height: u32) -> FrameBuffer32 {
        self.set_bpp(32);
        self.set_virt_width_height(width, height);
        self.set_phys_width_height(width, height);
        let (base_ptr, _size) = self.allocate_fb();
        FrameBuffer32 {
            base_ptr,
            width,
            height,
            stride: (width * 4) as usize,
        }
    }

    fn get_u32(&mut self, tag: u32, value: u32) -> u32 {
        let mut request = Request {
            size: core::mem::size_of::<GetSingleU32>() + 12,
            request: 0,
            payload: GetSingleU32 {
                tag_id: tag,
                value_buffer_size: 4,
                request_code: 0,
                buffer: [value],
            },
            end: 0,
        };

        self.exchange_request(&mut request);

        request.payload.buffer[0]
    }

    fn get_u32_pair(&mut self, tag: u32, value: (u32, u32)) -> (u32, u32) {
        let mut request = Request {
            size: core::mem::size_of::<GetSingleU32>() + 12,
            request: 0,
            payload: GetTwoU32 {
                tag_id: tag,
                value_buffer_size: 8,
                request_code: 0,
                buffer: [value.0, value.1],
            },
            end: 0,
        };

        self.exchange_request(&mut request);

        (request.payload.buffer[0], request.payload.buffer[1])
    }

    fn get_u64(&mut self, tag: u32) -> u64 {
        let (lower, upper) = self.get_u32_pair(tag, (0, 0));
        (u64::from(upper) << 32) | u64::from(lower)
    }

    fn exchange_request<P>(&mut self, request: &mut Request<P>)
    where
        P: core::fmt::Debug,
    {
        let req_ptr = make_ptr(request as *mut _ as *mut u32);
        // semihosting::println!("> req {:x?} at {:010p}", request, request);
        let msg = Message::builder()
            .with_channel(Channel::ArmProperty)
            .with_upper(req_ptr)
            .build();
        self.mailbox.write(msg);
        let _reply = self.mailbox.block_read_channel(Channel::ArmProperty);
        // semihosting::println!("< req {:x?} at {:010p}", request, request);
    }
}

#[repr(align(16), C)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct Request<P> {
    pub size: usize,
    pub request: u32,
    pub payload: P,
    pub end: u32,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetSingleU32 {
    pub tag_id: u32,
    pub value_buffer_size: u32,
    pub request_code: u32,
    pub buffer: [u32; 1],
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTwoU32 {
    pub tag_id: u32,
    pub value_buffer_size: u32,
    pub request_code: u32,
    pub buffer: [u32; 2],
}

fn make_ptr(ptr: *mut u32) -> u28 {
    if ptr.align_offset(16) != 0 {
        panic!("Misaligned pointer fed to VC!");
    }

    // L2 cache is enabled by default, so this is where VC seems Arm memory
    let ptr = 0x40000000 + (ptr as u32);

    u28::from_u32(ptr >> 4)
}
