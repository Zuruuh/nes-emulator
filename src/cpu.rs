use crate::bus::Bus;
use enumflags2::bitflags;

#[repr(u8)]
#[bitflags]
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum CPUFlag {
    #[default]
    CarryBit = 1 << 0,
    Zero = 1 << 1,
    DisableInterrupts = 1 << 2,
    DecimalMode = 1 << 3,
    Break = 1 << 4,
    _Unused = 1 << 5,
    Overflow = 1 << 6,
    Negative = 1 << 7,
}

#[derive(Copy, Clone, Debug)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    NoneAddressing,
}

#[derive(Default)]
pub struct CPU {
    bus: Bus,
    status: CPUFlag,
    register_a: u8,
    register_b: u8,
    register_y: u8,
    stack_ptr: u8,
    program_counter: u16,
    addr_mode: AddressingMode,
}

impl CPU {
    pub fn write(&mut self, addr: u16, data: u8) {
        self.bus.write(addr, data)
    }

    pub fn read(&self, addr: u16) -> Option<u8> {
        self.bus.read(addr)
    }

    pub fn addr_mode(&mut self, addr_mode: AddressingMode) {
        self.addr_mode = addr_mode;
    }
}
