//! Code for the Raspberry Pi VideoCore Mailbox

use arbitrary_int::u28;

#[bitbybit::bitenum(u4)]
#[derive(PartialEq, Debug)]
pub enum Channel {
    /// 0: Power management
    PowMan = 0,
    /// 1: Framebuffer
    Framebuffer = 1,
    /// 2: Virtual UART
    /// 3: VCHIQ
    /// 4: LEDs
    /// 5: Buttons
    /// 6: Touch screen
    /// 7:
    /// 8: Property tags (ARM -> VC)
    ArmProperty = 8,
    /// 9: Property tags (VC -> ARM)
    VcProperty = 9,
}

pub struct Driver {
    registers: MmioRegisters<'static>,
}

impl Driver {
    pub unsafe fn new() -> Self {
        Self {
            registers: unsafe { Registers::new_mmio_at(super::MMIO_BASE + 0xB880) },
        }
    }

    /// Non-blocking read from the mailbox
    pub fn read(&mut self) -> Option<Message> {
        if !self.registers.read_status().empty() {
            Some(self.registers.read_fifo_rx())
        } else {
            None
        }
    }

    /// Blocking read from the mailbox, for a specific channel
    pub fn block_read_channel(&mut self, channel: Channel) -> Message {
        loop {
            if let Some(message) = self.read() {
                if message.channel() == Ok(channel) {
                    return message;
                }
            }
        }
    }

    /// Blocking write to the mailbox
    pub fn write(&mut self, message: Message) {
        while self.registers.read_status().full() {
            core::hint::spin_loop();
        }
        self.registers.write_fifo_tx(message);
    }
}

#[repr(C)]
#[derive(derive_mmio::Mmio)]
pub struct Registers {
    #[mmio(Read)]
    fifo_rx: Message,
    _padding1: [u32; 5],
    #[mmio(Read)]
    status: Status,
    _padding2: [u32; 1],
    #[mmio(Write)]
    fifo_tx: Message,
}

#[bitbybit::bitfield(u32, default = 0, debug)]
pub struct Message {
    #[bits(4..=31, rw)]
    upper: u28,
    #[bits(0..=3, rw)]
    channel: Option<Channel>,
}

#[bitbybit::bitfield(u32, debug)]
pub struct Status {
    #[bit(31, r)]
    full: bool,
    #[bit(30, r)]
    empty: bool,
}
