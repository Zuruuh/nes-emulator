use super::{memory::Memory, Cpu};

const STACK: u16 = 0x0100;
const STACK_RESET: u8 = 0xfd;

pub trait Stack: Memory {
    fn stack_pop(&mut self) -> u8;

    fn stack_push(&mut self, data: u8);

    fn stack_push_u16(&mut self, data: u16);

    fn stack_pop_u16(&mut self) -> u16;
}

impl Stack for Cpu {
    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read((STACK as u16) + self.stack_pointer as u16)
    }

    fn stack_push(&mut self, data: u8) {
        self.mem_write((STACK as u16) + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1)
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        hi << 8 | lo
    }
}
