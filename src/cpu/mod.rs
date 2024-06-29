#![allow(unused)] // please leave me alone clippy

mod addressing_mode;
mod flags;

use addressing_mode::AddressingMode;
use enumflags2::BitFlags;
use flags::CpuFlags;

#[derive(Debug)]
pub struct Cpu {
    // accumulator
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: BitFlags<CpuFlags>,
    pub program_counter: u16,
    memory: [u8; 0xFFFF],
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: BitFlags::default(),
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }
}

const RESET_ADDRESS: u16 = 0xFFFC;

impl Cpu {
    pub fn get_operand_address(&self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,
            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPageX => {
                let pos = self.mem_read(self.program_counter);

                pos.wrapping_add(self.register_x) as u16
            }
            AddressingMode::ZeroPageY => {
                let pos = self.mem_read(self.program_counter);

                pos.wrapping_add(self.register_y) as u16
            }

            AddressingMode::AbsoluteX => {
                let base = self.mem_read_u16(self.program_counter);

                base.wrapping_add(self.register_x as u16)
            }
            AddressingMode::AbsoluteY => {
                let base = self.mem_read_u16(self.program_counter);

                base.wrapping_add(self.register_y as u16)
            }

            AddressingMode::IndirectX => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = (base as u8).wrapping_add(self.register_x);

                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::IndirectY => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }
            AddressingMode::NoneAddressing => todo!(),
        }
    }

    pub fn run(&mut self) {
        loop {
            let opcode = self.mem_read(self.program_counter);
            self.program_counter += 1;

            match opcode {
                // LDA https://www.nesdev.org/obelisk-6502-guide/reference.html#LDA
                0xA9 => {
                    self.lda(AddressingMode::Immediate);
                    self.program_counter += 1;
                }
                0xA5 => {
                    self.lda(AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0xB5 => {
                    self.lda(AddressingMode::ZeroPageX);
                    self.program_counter += 1;
                }
                0xAD => {
                    self.lda(AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0xBD => {
                    self.lda(AddressingMode::AbsoluteX);
                    self.program_counter += 2;
                }
                0xB9 => {
                    self.lda(AddressingMode::AbsoluteY);
                    self.program_counter += 2;
                }
                0xA1 => {
                    self.lda(AddressingMode::IndirectX);
                    self.program_counter += 1;
                }
                0xB1 => {
                    self.lda(AddressingMode::IndirectY);
                    self.program_counter += 1;
                }

                // STA https://www.nesdev.org/obelisk-6502-guide/reference.html#STA
                0x85 => {
                    self.sta(AddressingMode::ZeroPage);
                    self.program_counter += 1;
                }
                0x95 => {
                    self.sta(AddressingMode::ZeroPageX);
                    self.program_counter += 1;
                }
                0x8D => {
                    self.sta(AddressingMode::Absolute);
                    self.program_counter += 2;
                }
                0x9D => {
                    self.sta(AddressingMode::AbsoluteX);
                    self.program_counter += 2;
                }
                0x99 => {
                    self.sta(AddressingMode::AbsoluteY);
                    self.program_counter += 2;
                }
                0x81 => {
                    self.sta(AddressingMode::IndirectX);
                    self.program_counter += 1;
                }
                0x91 => {
                    self.sta(AddressingMode::IndirectY);
                    self.program_counter += 1;
                }

                0xAA => self.tax(),
                0xE8 => self.inx(),
                0x00 => return,
                _ => {}
            }
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.mem_read(addr), self.mem_read(addr + 1)])
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let data = data.to_le_bytes();

        self.mem_write(addr, data[0]);
        self.mem_write(addr + 1, data[1]);
    }

    fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = BitFlags::default();

        self.program_counter = self.mem_read_u16(RESET_ADDRESS);
    }

    fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program[..]);
        self.mem_write_u16(RESET_ADDRESS, 0x8000);
    }

    fn lda(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn sta(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;

        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        dbg!(self.register_x);
        self.register_x = self.register_x.wrapping_add(1);
    }

    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.status.set(CpuFlags::Zero, result == 0);

        self.status.set(
            CpuFlags::Negative,
            result & CpuFlags::Negative.into_bitflags().bits() != 0,
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = Cpu::default();
        cpu.load_and_run(vec![0xA9, 0x05, 0x00]);
        dbg!(&cpu.status);

        assert_eq!(cpu.register_a, 0x05);
        assert!(!cpu.status.contains(CpuFlags::Zero));
        assert!(!cpu.status.contains(CpuFlags::Negative));
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = Cpu::default();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);

        assert_eq!(cpu.register_a, 0);
        assert!(cpu.status.contains(CpuFlags::Zero));
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = Cpu::default();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = Cpu::default();
        cpu.load(vec![0xAA, 0x00]);
        cpu.reset();
        cpu.register_a = 10;
        cpu.run();

        assert_eq!(cpu.register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = Cpu::default();
        cpu.load_and_run(vec![0xA9, 0xC0, 0xAA, 0xE8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = Cpu::default();
        cpu.load(vec![0xE8, 0xE8, 0x00]);
        cpu.reset();
        cpu.register_x = 0xFF;
        cpu.run();

        assert_eq!(cpu.register_x, 1)
    }
}
