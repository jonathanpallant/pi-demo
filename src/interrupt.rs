/// The interrupts on the BCM2835
pub enum Interrupt {
    Aux = 29,
    Uart = 32 + 25,
}

pub struct Driver {
    registers: MmioRegisters<'static>,
}

impl Driver {
    pub const unsafe fn new() -> Self {
        Self {
            registers: unsafe { Registers::new_mmio_at(super::MMIO_BASE + 0xB200) },
        }
    }

    /// Disable (mask) all interrupts
    pub fn disable_irq_all(&mut self) {
        self.registers.write_disable_irqs_1(0xFFFF_FFFF);
        self.registers.write_disable_irqs_2(0xFFFF_FFFF);
        self.registers.write_disable_irqs_basic(0xFFF);
    }

    /// Enable (unmask) the given interrupt
    pub fn enable_irq(&mut self, interrupt: Interrupt) {
        let number = interrupt as u8;
        if let Some(lo_number) = number.checked_sub(32) {
            self.registers.write_enable_irqs_2(1 << lo_number);
        } else {
            self.registers.write_enable_irqs_1(1 << number);
        }
    }
}

#[derive(derive_mmio::Mmio)]
#[repr(C)]
struct Registers {
    basic_pending: u32,
    irq_pending_1: u32,
    irq_pending_2: u32,
    fiq_control: u32,
    #[mmio(PureRead, Write)]
    enable_irqs_1: u32,
    #[mmio(PureRead, Write)]
    enable_irqs_2: u32,
    #[mmio(PureRead, Write)]
    enable_irqs_basic: u32,
    #[mmio(Write)]
    disable_irqs_1: u32,
    #[mmio(Write)]
    disable_irqs_2: u32,
    #[mmio(Write)]
    disable_irqs_basic: u32,
}
