//! IBM 1130 CPU state.
//!
//! All registers are 16 bits. ACC and EXT pair as a 32-bit value
//! for double-precision ops. XR1..XR3 are the index registers. IAR
//! is the program counter. Carry and Overflow are sticky condition
//! flags maintained by arith ops; BSC reads them.

#[derive(Debug, Clone, Copy, Default)]
pub struct CpuState {
    pub acc: u16,
    pub ext: u16,
    pub xr1: u16,
    pub xr2: u16,
    pub xr3: u16,
    pub iar: u16,
    pub carry: bool,
    pub overflow: bool,
    pub halted: bool,
    /// Instruction count -- useful for guarding against runaway
    /// programs in tests.
    pub instr_count: u64,
}

impl CpuState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Read the index register selected by `tag` (1..=3). Tag 0
    /// means "no XR"; callers handle it explicitly. Returns 0 for
    /// unknown tags so that `effective_address(addr, 0)` is just
    /// `addr`.
    pub fn read_xr(&self, tag: u8) -> u16 {
        match tag {
            1 => self.xr1,
            2 => self.xr2,
            3 => self.xr3,
            _ => 0,
        }
    }

    pub fn write_xr(&mut self, tag: u8, v: u16) {
        match tag {
            1 => self.xr1 = v,
            2 => self.xr2 = v,
            3 => self.xr3 = v,
            _ => {}
        }
    }

    /// 32-bit pair (ACC high, EXT low) used by M/D/LDD/STD.
    pub fn read_pair(&self) -> i32 {
        ((self.acc as u32) << 16 | self.ext as u32) as i32
    }

    pub fn write_pair(&mut self, v: i32) {
        let u = v as u32;
        self.acc = (u >> 16) as u16;
        self.ext = (u & 0xFFFF) as u16;
    }
}
