use enumflags2::{bitflags, BitFlags};

#[repr(u8)]
#[bitflags]
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum CpuFlags {
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

impl CpuFlags {
    pub fn into_bitflags(self) -> BitFlags<Self> {
        BitFlags::from(self)
    }
}
