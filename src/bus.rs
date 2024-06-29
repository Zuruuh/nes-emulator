pub struct Bus {
    memory: [u8; 0xFFFF],
}
impl Default for Bus {
    fn default() -> Self {
        Self {
            memory: [0; 0xFFFF],
        }
    }
}

impl Bus {
    pub fn write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    pub fn read(&self, addr: u16) -> Option<u8> {
        self.memory.get(addr as usize).copied()
    }
}
