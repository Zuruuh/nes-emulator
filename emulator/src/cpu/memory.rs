use tracing::instrument;

use super::{addressing_mode::AddressingMode, Cpu};

pub trait Memory {
    fn mem_read(&self, addr: u16) -> u8;

    fn mem_write(&mut self, addr: u16, data: u8);

    fn mem_read_u16(&self, addr: u16) -> u16 {
        u16::from_le_bytes([self.mem_read(addr), self.mem_read(addr + 1)])
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let data = data.to_le_bytes();

        self.mem_write(addr, data[0]);
        self.mem_write(addr + 1, data[1]);
    }
    fn get_operand_address(&self, mode: AddressingMode) -> u16;
}

impl Memory for Cpu {
    #[instrument]
    fn mem_read(&self, addr: u16) -> u8 {
        log::debug!("Reading memory address {:X?}", addr);
        self.memory[addr as usize]
    }

    #[instrument]
    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
        log::trace!("Writing {:X?} at {:X?}", data, addr);
    }

    fn get_operand_address(&self, mode: AddressingMode) -> u16 {
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
}
