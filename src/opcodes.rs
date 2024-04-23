#![allow(unused)] // please leave me alone clippy

use std::collections::BTreeMap;

use crate::cpu::AddressingMode;

use once_cell::sync::Lazy;

pub static OPCODES: Lazy<Vec<Opcode>> = Lazy::new(|| {
    vec![
        // see https://www.nesdev.org/obelisk-6502-guide/reference.html#BRK
        Opcode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
        // see https://www.nesdev.org/obelisk-6502-guide/reference.html#TAX
        Opcode::new(0xaa, "TAX", 1, 2, AddressingMode::NoneAddressing),
        // see https://www.nesdev.org/obelisk-6502-guide/reference.html#INX
        Opcode::new(0xe8, "INX", 1, 2, AddressingMode::NoneAddressing),
        // see https://www.nesdev.org/obelisk-6502-guide/reference.html#LDA
        Opcode::new(0xa9, "LDA", 2, 2, AddressingMode::Immediate),
        Opcode::new(0xa5, "LDA", 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0xb5, "LDA", 2, 4, AddressingMode::ZeroPage_X),
        Opcode::new(0xad, "LDA", 3, 4, AddressingMode::Absolute),
        Opcode::new(
            0xbd,
            "LDA",
            3,
            4, /*+1 if page crossed*/
            AddressingMode::Absolute_X,
        ),
        Opcode::new(
            0xb9,
            "LDA",
            3,
            4, /*+1 if page crossed*/
            AddressingMode::Absolute_Y,
        ),
        Opcode::new(0xa1, "LDA", 2, 6, AddressingMode::Indirect_X),
        Opcode::new(
            0xb1,
            "LDA",
            2,
            5, /*+1 if page crossed*/
            AddressingMode::Indirect_Y,
        ),
        // see https://www.nesdev.org/obelisk-6502-guide/reference.html#STA
        Opcode::new(0x85, "STA", 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x95, "STA", 2, 4, AddressingMode::ZeroPage_X),
        Opcode::new(0x8d, "STA", 3, 4, AddressingMode::Absolute),
        Opcode::new(0x9d, "STA", 3, 5, AddressingMode::Absolute_X),
        Opcode::new(0x99, "STA", 3, 5, AddressingMode::Absolute_Y),
        Opcode::new(0x81, "STA", 2, 6, AddressingMode::Indirect_X),
        Opcode::new(0x91, "STA", 2, 6, AddressingMode::Indirect_Y),
        // see https://www.nesdev.org/obelisk-6502-guide/reference.html#ADC
        Opcode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate),
        Opcode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage),
        Opcode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPage_X),
        Opcode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute),
        Opcode::new(0x7D, "ADC", 3, 4, AddressingMode::Absolute_X),
        Opcode::new(0x79, "ADC", 3, 4, AddressingMode::Absolute_Y),
        Opcode::new(0x61, "ADC", 2, 6, AddressingMode::Indirect_X),
        Opcode::new(0x71, "ADC", 2, 5, AddressingMode::Indirect_X),
    ]
});

pub static OPCODES_MAP: Lazy<BTreeMap<u8, &'static Opcode>> =
    Lazy::new(|| OPCODES.iter().map(|opcode| (opcode.code, opcode)).collect());

#[derive(Debug, Copy, Clone)]
pub struct Opcode {
    code: u8,
    repr: &'static str,
    bytes: u8,
    cycles: u8,
    mode: AddressingMode,
}

impl Opcode {
    pub fn new(code: u8, repr: &'static str, bytes: u8, cycles: u8, mode: AddressingMode) -> Self {
        Self {
            code,
            repr,
            bytes,
            cycles,
            mode,
        }
    }

    pub fn mode(&self) -> &AddressingMode {
        &self.mode
    }
}
