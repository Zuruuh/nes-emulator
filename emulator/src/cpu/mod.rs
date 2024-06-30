#![allow(unused)] // please leave me alone clippy

pub mod addressing_mode;
pub mod flags;
pub mod memory;
pub mod opcode;
pub mod stack;

use enumflags2::BitFlags;

use addressing_mode::AddressingMode;
use flags::CpuFlags;
use memory::Memory;
use opcode::OPCODES_MAP;
use stack::Stack;
use tracing::instrument;

#[derive(Debug)]
pub struct Cpu {
    // accumulator
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: BitFlags<CpuFlags>,
    pub program_counter: u16,
    pub stack_pointer: u8,
    memory: [u8; u16::MAX as usize],
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: BitFlags::default(),
            program_counter: 0,
            stack_pointer: 0,
            memory: [0; u16::MAX as usize],
        }
    }
}

const RESET_ADDRESS: u16 = 0xFFFC;
const GAME_START_ADDRESS: u16 = 0x0600;

pub enum RunResult {
    Running,
    Done,
}

impl Cpu {
    pub fn run(&mut self) {
        loop {
            match self.run_cycle_with_callback(|_| {}) {
                RunResult::Running => {}
                RunResult::Done => break,
            }
        }
    }

    pub fn run_cycle_with_callback<F>(&mut self, mut callback: F) -> RunResult
    where
        F: FnMut(&mut Cpu),
    {
        callback(self);
        let opcode = self.mem_read(self.program_counter);
        self.program_counter += 1;

        let opcode = (*&OPCODES_MAP).get(&opcode).copied().expect(&format!(
            "Illegal opcode instruction provided {:X?}",
            opcode
        ));

        log::debug!("Executing instruction {:?}", &opcode);

        match opcode.repr {
            "ADC" => self.adc(opcode.mode),
            "AND" => self.and(opcode.mode),
            "ASL" => self.asl(opcode.mode),
            "BCC" => self.branch(!self.status.contains(CpuFlags::CarryBit)),
            "BCS" => self.branch(self.status.contains(CpuFlags::CarryBit)),
            "BEQ" => self.branch(self.status.contains(CpuFlags::Zero)),
            "BIT" => self.bit(opcode.mode),
            "BMI" => self.branch(self.status.contains(CpuFlags::Negative)),
            "BNE" => self.branch(!self.status.contains(CpuFlags::Zero)),
            "BPL" => self.branch(!self.status.contains(CpuFlags::Negative)),
            "BRK" => return RunResult::Done,
            "BVC" => self.branch(!self.status.contains(CpuFlags::Overflow)),
            "BVS" => self.branch(self.status.contains(CpuFlags::Overflow)),
            "CLC" => self.status.remove(CpuFlags::CarryBit),
            "CLD" => self.status.remove(CpuFlags::DecimalMode),
            "CLI" => self.status.remove(CpuFlags::DisableInterrupts),
            "CLV" => self.status.remove(CpuFlags::Overflow),
            "CMP" => self.compare(opcode.mode, self.register_a),
            "CPX" => self.compare(opcode.mode, self.register_x),
            "CPY" => self.compare(opcode.mode, self.register_y),
            "DEC" => self.dec(),
            "DEX" => self.dex(),
            "DEY" => self.dey(),
            "EOR" => self.eor(opcode.mode),
            "INC" => self.inc(),
            "INX" => self.inx(),
            "INY" => self.iny(),
            "JMP" => match opcode.code {
                0x6c => {
                    let mem_address = self.mem_read_u16(self.program_counter);
                    // let indirect_ref = self.mem_read_u16(mem_address);
                    // 6502 bug mode with with page boundary:
                    // if address $3000 contains $40, $30FF contains $80, and $3100 contains $50,
                    // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended
                    // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000
                    //
                    // See https://www.nesdev.org/obelisk-6502-guide/reference.html#JMP for ref

                    let indirect_ref = if mem_address & 0x00FF == 0x00FF {
                        let lo = self.mem_read(mem_address);
                        let hi = self.mem_read(mem_address & 0xFF00);
                        (hi as u16) << 8 | (lo as u16)
                    } else {
                        self.mem_read_u16(mem_address)
                    };

                    self.program_counter = indirect_ref;
                }
                _ => {
                    let addr = self.mem_read_u16(self.program_counter);
                    self.program_counter = addr;
                }
            },
            "JSR" => {
                self.stack_push_u16(self.program_counter + 2 - 1);
                let target = self.mem_read_u16(self.program_counter);
                self.program_counter = target;
            }
            "LDA" => self.lda(opcode.mode),
            "LDX" => self.ldx(opcode.mode),
            "LDY" => self.ldy(opcode.mode),
            "LSR" => match opcode.code {
                0x4A => self.lsr_accumulator(),
                _ => self.lsr(opcode.mode),
            },
            "NOP" => {}
            "ORA" => self.ora(opcode.mode),
            "PHA" => self.stack_push(self.register_a),
            "PHP" => self.php(),
            "PLA" => self.pla(),
            "PLP" => self.plp(),
            "ROL" => match opcode.code {
                0x2A => self.rol_accumulator(),
                _ => self.rol(opcode.mode),
            },
            "ROR" => match opcode.code {
                0x6A => self.ror_accumulator(),
                _ => self.ror(opcode.mode),
            },
            "RTI" => self.rti(),
            "RTS" => self.program_counter = self.stack_pop_u16() + 1,
            "SBC" => self.sbc(opcode.mode),
            "SEC" => self.status.insert(CpuFlags::CarryBit),
            "SED" => self.status.insert(CpuFlags::DecimalMode),
            "SEI" => self.status.insert(CpuFlags::DisableInterrupts),
            "STA" => self.sta(opcode.mode),
            "STX" => self.stx(opcode.mode),
            "STY" => self.sty(opcode.mode),
            "TAX" => self.tax(),
            "TAY" => self.tay(),
            "TSX" => self.tsx(),
            "TXA" => self.txa(),
            "TXS" => self.txs(),
            "TYA" => self.tya(),

            _ => unreachable!(
                "Invalid byte {:X?} - Dumping memory: {:?}",
                opcode.repr, self.memory
            ),
        }

        self.program_counter += opcode.len as u16 - 1;

        RunResult::Running
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.status = BitFlags::default();

        self.program_counter = self.mem_read_u16(RESET_ADDRESS);
    }

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[(GAME_START_ADDRESS as usize)..(GAME_START_ADDRESS as usize + program.len())]
            .copy_from_slice(&program[..]);
        self.mem_write_u16(RESET_ADDRESS, GAME_START_ADDRESS);
    }

    #[instrument]
    fn adc(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);

        self.add_to_register_a(self.mem_read(addr));
    }

    #[instrument]
    fn asl(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        self.status.set(CpuFlags::CarryBit, data >> 7 == 1);
        data = data << 1;

        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    #[instrument]
    fn and(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        self.set_register_a(data & self.register_a);
    }

    #[instrument]
    fn branch(&mut self, condition: bool) {
        if !condition {
            return;
        }

        let jump: i8 = self.mem_read(self.program_counter) as i8;
        let jump_addr = self
            .program_counter
            .wrapping_add(1)
            .wrapping_add(jump as u16);

        self.program_counter = jump_addr;
    }

    #[instrument]
    fn bit(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        let and = self.register_a & data;
        self.status.set(CpuFlags::Zero, and == 0);

        self.status.set(
            CpuFlags::Negative,
            data & CpuFlags::Negative.into_bitflags().bits() > 0,
        );
        self.status.set(
            CpuFlags::Overflow,
            data & CpuFlags::Overflow.into_bitflags().bits() > 0,
        );
    }

    #[instrument]
    fn compare(&mut self, mode: AddressingMode, compare_with: u8) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        self.status.set(CpuFlags::CarryBit, data <= compare_with);

        self.update_zero_and_negative_flags(compare_with.wrapping_sub(data));
    }

    #[instrument]
    fn dec(&mut self) {
        self.register_a = self.register_a.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_a);
    }

    #[instrument]
    fn dex(&mut self) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    #[instrument]
    fn dey(&mut self) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    #[instrument]
    fn eor(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.set_register_a(data ^ self.register_a);
    }

    #[instrument]
    fn inc(&mut self) {
        self.register_a = self.register_a.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_a);
    }

    #[instrument]
    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }

    #[instrument]
    fn iny(&mut self) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_y);
    }

    #[instrument]
    fn lda(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);

        self.set_register_a(value);
    }

    #[instrument]
    fn ldx(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.mem_read(addr);
        self.register_x = self.mem_read(addr);

        self.update_zero_and_negative_flags(self.register_x);
    }

    #[instrument]
    fn ldy(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.register_x = self.mem_read(addr);

        self.update_zero_and_negative_flags(self.register_y);
    }

    #[instrument]
    fn lsr_accumulator(&mut self) {
        let mut data = self.register_a;
        self.status.set(CpuFlags::CarryBit, data & 1 == 1);
        data = data >> 1;
        self.set_register_a(data);
    }

    #[instrument]
    fn lsr(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);

        self.status.set(CpuFlags::CarryBit, data & 1 == 1);
        data = data >> 1;
        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    #[instrument]
    fn ora(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);
        self.set_register_a(data | self.register_a);
    }

    #[instrument]
    fn php(&mut self) {
        let mut status = self.status.clone();
        status.insert(CpuFlags::Break);
        status.insert(CpuFlags::_Unused);

        self.stack_push(status.bits());
    }

    #[instrument]
    fn pla(&mut self) {
        let value = self.stack_pop();
        self.set_register_a(value);
    }

    #[instrument]
    fn plp(&mut self) {
        self.pop_status_from_stack();
        self.status.remove(CpuFlags::Break);
        self.status.insert(CpuFlags::_Unused);
    }

    #[instrument]
    fn rol_accumulator(&mut self) {
        let mut data = self.register_a;
        let old_carry = self.status.contains(CpuFlags::CarryBit);

        self.status.set(CpuFlags::CarryBit, data >> 7 == 1);
        data = data << 1;
        if old_carry {
            data = data | 1;
        }

        self.set_register_a(data);
    }

    #[instrument]
    fn rol(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        let old_carry = self.status.contains(CpuFlags::CarryBit);

        self.status.set(CpuFlags::CarryBit, data >> 7 == 1);
        data = data << 1;
        if old_carry {
            data = data | 1;
        }

        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    #[instrument]
    fn ror_accumulator(&mut self) {
        let mut data = self.register_a;
        let old_carry = self.status.contains(CpuFlags::CarryBit);

        self.status.set(CpuFlags::CarryBit, data & 1 == 1);
        data = data >> 1;
        if old_carry {
            data = data | CpuFlags::Negative.into_bitflags().bits();
        }

        self.set_register_a(data);
    }

    #[instrument]
    fn ror(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut data = self.mem_read(addr);
        let old_carry = self.status.contains(CpuFlags::CarryBit);

        self.status.set(CpuFlags::CarryBit, data & 1 == 1);
        data = data >> 1;
        if old_carry {
            data = data | CpuFlags::Negative.into_bitflags().bits();
        }

        self.mem_write(addr, data);
        self.update_zero_and_negative_flags(data);
    }

    #[instrument]
    fn rti(&mut self) {
        self.pop_status_from_stack();
        self.status.remove(CpuFlags::Break);
        self.status.insert(CpuFlags::_Unused);

        self.program_counter = self.stack_pop_u16();
    }

    #[instrument]
    fn sbc(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let data = self.mem_read(addr);

        self.add_to_register_a((data as i8).wrapping_neg().wrapping_sub(1) as u8)
    }

    #[instrument]
    fn sta(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_a);
    }

    #[instrument]
    fn stx(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_x);
    }

    #[instrument]
    fn sty(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.mem_write(addr, self.register_y);
    }

    #[instrument]
    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    #[instrument]
    fn tay(&mut self) {
        self.register_y = self.register_a;
        self.update_zero_and_negative_flags(self.register_y);
    }

    #[instrument]
    fn tsx(&mut self) {
        self.register_x = self.stack_pointer;
        self.update_zero_and_negative_flags(self.register_x);
    }

    #[instrument]
    fn txa(&mut self) {
        self.register_a = self.register_x;
        self.update_zero_and_negative_flags(self.register_a);
    }

    #[instrument]
    fn txs(&mut self) {
        self.stack_pointer = self.register_x;
    }

    #[instrument]
    fn tya(&mut self) {
        self.register_a = self.register_y;
        self.update_zero_and_negative_flags(self.register_a);
    }

    #[instrument]
    fn update_zero_and_negative_flags(&mut self, result: u8) {
        self.status.set(CpuFlags::Zero, result == 0);

        self.status.set(
            CpuFlags::Negative,
            result & CpuFlags::Negative.into_bitflags().bits() != 0,
        );
    }

    fn set_register_a(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn add_to_register_a(&mut self, value: u8) {
        let sum = self.register_a as u16
            + value as u16
            + self
                .status
                .contains(CpuFlags::CarryBit)
                .then_some(1u16)
                .unwrap_or_default();

        self.status.set(CpuFlags::CarryBit, sum > u8::MAX as u16);

        let result = sum as u8;

        self.status.set(
            CpuFlags::Overflow,
            (value ^ result) & (result ^ self.register_a) & 0x80 != 0,
        );

        self.set_register_a(result);
    }

    fn pop_status_from_stack(&mut self) {
        self.status = BitFlags::from_bits(self.stack_pop())
            .expect("Could not deserialize bits from stack into status flags");
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
        cpu.register_x = u8::MAX;
        cpu.run();

        assert_eq!(cpu.register_x, 1)
    }

    #[test]
    fn test_branch_timings() {
        let mut cpu = Cpu::default();
        let bytes = include_bytes!("../../branch_timing_tests/1.Branch_Basics.nes").to_vec();
        cpu.load(bytes);
        cpu.reset();

        cpu.run();
    }
}
