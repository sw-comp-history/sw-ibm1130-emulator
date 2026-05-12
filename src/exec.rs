//! Instruction execution: one `step()` per instruction.
//!
//! BSC / BSI condition mask semantics (saga step 3 of forth-on-1130
//! aligned the bit assignments with Moore's authoritative 1968
//! FORTH listing; sw-ibm1130-isa's spec was updated to expose the
//! mask field, closing the postmortem-Sec-4 gap).
//!
//! ```text
//! mask bit | meaning when set
//! ---------+-----------------------------------------------------
//! 0x04     | E   -- ACC even (low bit clear)
//! 0x08     | +   -- ACC > 0
//! 0x10     | -   -- ACC < 0 (two's-complement)
//! 0x20     | Z   -- ACC == 0
//! 0x40     | C   -- carry indicator set
//! ```
//!
//! Bits 0x01, 0x02, and 0x80 are unused. Bit assignments follow
//! Moore's FORTH-listing constants (`:EVEN 04`, `:POSITIVE 8`,
//! `:NEGATIVE 10`, `:EQUAL 20`).
//!
//! **BSC short** (`BSC mask`): tests the displacement byte as the
//! mask. If any masked condition matches, **skip** the next
//! instruction (advance IAR by that instruction's word size).
//! Mask = 0 means no condition tested -> never skip.
//!
//! **BSC long** (`BSC L target [, mask]`): branches to `target` if
//! `mask == 0` (the "always" case) or if any masked condition
//! holds. Otherwise falls through. This is the natural mapping
//! for `B` (mask=0 -> always), `BZ` (mask=0x20 -> if Z), `BN`
//! (mask=0x10 -> if N), etc.
//!
//! **BSI long** (`BSI L target [, mask]`): same condition logic as
//! BSC long, but on "take" performs a subroutine call (writes IAR
//! into target, sets IAR = target+1). Used by Moore's
//! `:CONDITION` family to thread conditional calls through NEXT.
//!
//! ACC sign is tested in **two's-complement** terms (the natural
//! interpretation; the historical 1130 used sign-magnitude, which
//! diverges only at the value 0x8000 for `+` vs `-`).

use sw_isa_core::DecodeError;

use crate::memory::Memory;
use crate::state::CpuState;
use sw_ibm1130_isa::{Instruction, Opcode};

/// Errors a step() can produce.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecError {
    Halted,
    Decode(String),
    Unsupported(Opcode),
    DivideByZero,
}

impl From<DecodeError> for ExecError {
    fn from(e: DecodeError) -> Self {
        ExecError::Decode(format!("{e:?}"))
    }
}

/// Decode + execute one instruction. Advances IAR; halts on WAIT.
pub fn step(state: &mut CpuState, mem: &mut Memory) -> Result<(), ExecError> {
    if state.halted {
        return Err(ExecError::Halted);
    }
    let (insn, size_words) = mem.decode_at(state.iar)?;
    state.iar = state.iar.wrapping_add(size_words);
    state.instr_count += 1;
    match insn {
        Instruction::Short { op, tag, disp } => exec_short(state, mem, op, tag, disp),
        Instruction::Long {
            op,
            tag,
            indirect,
            mask,
            address,
        } => exec_long(state, mem, op, tag, indirect, mask, address),
    }
}

/// Run until the CPU halts (Wait instruction) or `max_steps` is
/// reached. Returns the number of instructions executed.
pub fn run(state: &mut CpuState, mem: &mut Memory, max_steps: u64) -> Result<u64, ExecError> {
    let start = state.instr_count;
    while !state.halted && state.instr_count - start < max_steps {
        step(state, mem)?;
    }
    Ok(state.instr_count - start)
}

fn effective_address(state: &CpuState, tag: u8, base: u16) -> u16 {
    base.wrapping_add(state.read_xr(tag))
}

/// Operand for memory-reference ops: word at the effective address,
/// with optional indirection (long form only).
fn fetch_operand(state: &CpuState, mem: &Memory, tag: u8, address: u16, indirect: bool) -> u16 {
    let ea = effective_address(state, tag, address);
    let direct = mem.read_word(ea);
    if indirect {
        mem.read_word(direct)
    } else {
        direct
    }
}

/// Address used as a *destination* by store-style ops -- after
/// indirection, this is the word we modify.
fn destination_address(
    state: &CpuState,
    mem: &Memory,
    tag: u8,
    address: u16,
    indirect: bool,
) -> u16 {
    let ea = effective_address(state, tag, address);
    if indirect { mem.read_word(ea) } else { ea }
}

fn exec_short(
    state: &mut CpuState,
    mem: &mut Memory,
    op: Opcode,
    tag: u8,
    disp: i8,
) -> Result<(), ExecError> {
    // Short-form effective address is `IAR + disp + XR[tag]`. IAR
    // has already been advanced past this instruction.
    let pc_relative = state.iar.wrapping_add(disp as i16 as u16);
    let address = pc_relative;
    exec_with_address(state, mem, op, tag, false, address, ShortShape { disp })
}

#[derive(Copy, Clone)]
struct ShortShape {
    disp: i8,
}

fn exec_long(
    state: &mut CpuState,
    mem: &mut Memory,
    op: Opcode,
    tag: u8,
    indirect: bool,
    mask: u8,
    address: u16,
) -> Result<(), ExecError> {
    exec_with_address(state, mem, op, tag, indirect, address, LongShape { mask })
}

#[derive(Copy, Clone)]
struct LongShape {
    mask: u8,
}

trait FormShape: Copy {
    fn is_long(self) -> bool;
    fn short_disp(self) -> i8;
    /// Condition mask, for long-form BSC / BSI. Returns 0 for short
    /// form (which carries no mask of its own; the short form's
    /// `disp` byte IS the mask -- callers use `short_disp()` to
    /// read it).
    fn long_mask(self) -> u8;
}

impl FormShape for ShortShape {
    fn is_long(self) -> bool {
        false
    }
    fn short_disp(self) -> i8 {
        self.disp
    }
    fn long_mask(self) -> u8 {
        0
    }
}

impl FormShape for LongShape {
    fn is_long(self) -> bool {
        true
    }
    fn short_disp(self) -> i8 {
        0
    }
    fn long_mask(self) -> u8 {
        self.mask
    }
}

fn exec_with_address<S: FormShape>(
    state: &mut CpuState,
    mem: &mut Memory,
    op: Opcode,
    tag: u8,
    indirect: bool,
    address: u16,
    shape: S,
) -> Result<(), ExecError> {
    match op {
        Opcode::Load => {
            state.acc = fetch_operand(state, mem, tag, address, indirect);
            Ok(())
        }
        Opcode::LoadDouble => {
            let ea = effective_address(state, tag, address);
            let ea = if indirect { mem.read_word(ea) } else { ea };
            state.acc = mem.read_word(ea);
            state.ext = mem.read_word(ea.wrapping_add(1));
            Ok(())
        }
        Opcode::Store => {
            let dest = destination_address(state, mem, tag, address, indirect);
            mem.write_word(dest, state.acc);
            Ok(())
        }
        Opcode::StoreDouble => {
            let dest = destination_address(state, mem, tag, address, indirect);
            mem.write_word(dest, state.acc);
            mem.write_word(dest.wrapping_add(1), state.ext);
            Ok(())
        }
        Opcode::LoadIndex => {
            // LDX loads an index register from address. Tag selects
            // which XR (1..=3); tag 0 with LDX historically loaded
            // an immediate, but our codegen never emits that shape
            // and we treat tag 0 as a no-op load into "no XR".
            let v = mem.read_word(effective_address(state, 0, address));
            state.write_xr(tag, v);
            Ok(())
        }
        Opcode::StoreIndex => {
            let dest = destination_address(state, mem, 0, address, indirect);
            mem.write_word(dest, state.read_xr(tag));
            Ok(())
        }
        Opcode::LoadStatus | Opcode::StoreStatus => {
            // Status word handling deferred -- not exercised by the
            // step-11 demos; treat as no-op so programs that
            // happen to use them don't crash.
            Ok(())
        }
        Opcode::Add => {
            let operand = fetch_operand(state, mem, tag, address, indirect);
            let (result, carry) = state.acc.overflowing_add(operand);
            let signed_overflow = (state.acc as i16).checked_add(operand as i16).is_none();
            state.acc = result;
            if carry {
                state.carry = true;
            }
            if signed_overflow {
                state.overflow = true;
            }
            Ok(())
        }
        Opcode::AddDouble => {
            let ea = effective_address(state, tag, address);
            let ea = if indirect { mem.read_word(ea) } else { ea };
            let hi = mem.read_word(ea);
            let lo = mem.read_word(ea.wrapping_add(1));
            let operand = ((hi as u32) << 16 | lo as u32) as i32;
            let pair = state.read_pair();
            state.write_pair(pair.wrapping_add(operand));
            Ok(())
        }
        Opcode::Subtract => {
            let operand = fetch_operand(state, mem, tag, address, indirect);
            let (result, borrow) = state.acc.overflowing_sub(operand);
            let signed_overflow = (state.acc as i16).checked_sub(operand as i16).is_none();
            state.acc = result;
            if borrow {
                state.carry = true;
            }
            if signed_overflow {
                state.overflow = true;
            }
            Ok(())
        }
        Opcode::SubtractDouble => {
            let ea = effective_address(state, tag, address);
            let ea = if indirect { mem.read_word(ea) } else { ea };
            let hi = mem.read_word(ea);
            let lo = mem.read_word(ea.wrapping_add(1));
            let operand = ((hi as u32) << 16 | lo as u32) as i32;
            let pair = state.read_pair();
            state.write_pair(pair.wrapping_sub(operand));
            Ok(())
        }
        Opcode::Multiply => {
            let operand = fetch_operand(state, mem, tag, address, indirect) as i16 as i32;
            let acc = state.acc as i16 as i32;
            let product = acc.wrapping_mul(operand);
            state.write_pair(product);
            Ok(())
        }
        Opcode::Divide => {
            let divisor = fetch_operand(state, mem, tag, address, indirect) as i16 as i32;
            if divisor == 0 {
                return Err(ExecError::DivideByZero);
            }
            let dividend = state.read_pair();
            state.acc = (dividend / divisor) as u16;
            state.ext = (dividend % divisor) as u16;
            Ok(())
        }
        Opcode::And => {
            let operand = fetch_operand(state, mem, tag, address, indirect);
            state.acc &= operand;
            Ok(())
        }
        Opcode::Or => {
            let operand = fetch_operand(state, mem, tag, address, indirect);
            state.acc |= operand;
            Ok(())
        }
        Opcode::ExclusiveOr => {
            let operand = fetch_operand(state, mem, tag, address, indirect);
            state.acc ^= operand;
            Ok(())
        }
        Opcode::ShiftLeft => {
            let count = (shape.short_disp() as u8) & 0x3F;
            state.acc = state.acc.wrapping_shl(count as u32);
            Ok(())
        }
        Opcode::ShiftRight => {
            let count = (shape.short_disp() as u8) & 0x3F;
            // Arithmetic shift right (sign-preserving) is the
            // natural semantics for SRA in our minimal scope; SLT/
            // SLCA / sub-op variants are out of scope.
            let signed = state.acc as i16;
            state.acc = signed.wrapping_shr(count as u32) as u16;
            Ok(())
        }
        Opcode::BranchSkipCondition => exec_bsc(state, mem, tag, indirect, address, shape),
        Opcode::BranchStore => {
            // BSI: same condition semantics as BSC long. If the
            // condition test passes (mask == 0 OR any masked
            // condition matches), perform the call: write IAR to
            // target, set IAR = target+1. Otherwise fall through.
            let mask = shape.long_mask();
            if mask == 0 || condition_matches(state, mask) {
                let target = destination_address(state, mem, tag, address, indirect);
                mem.write_word(target, state.iar);
                state.iar = target.wrapping_add(1);
            }
            Ok(())
        }
        Opcode::ModifyIndex => exec_mdx(state, mem, tag, indirect, address, shape),
        Opcode::Wait => {
            state.halted = true;
            Ok(())
        }
        Opcode::ExecuteInputOutput => exec_xio(state, mem, tag, address, indirect),
    }
}

fn exec_bsc<S: FormShape>(
    state: &mut CpuState,
    mem: &Memory,
    tag: u8,
    indirect: bool,
    address: u16,
    shape: S,
) -> Result<(), ExecError> {
    if shape.is_long() {
        let mask = shape.long_mask();
        // mask == 0 => always branch; mask != 0 => branch if any
        // masked condition holds.
        if mask == 0 || condition_matches(state, mask) {
            let target = destination_address(state, mem, tag, address, indirect);
            state.iar = target;
        }
    } else {
        let mask = shape.short_disp() as u8;
        if condition_matches(state, mask) {
            // Short form: skip next instruction (1 or 2 words).
            if let Ok((_, size)) = mem.decode_at(state.iar) {
                state.iar = state.iar.wrapping_add(size);
            }
        }
    }
    Ok(())
}

fn exec_mdx<S: FormShape>(
    state: &mut CpuState,
    mem: &mut Memory,
    tag: u8,
    indirect: bool,
    address: u16,
    shape: S,
) -> Result<(), ExecError> {
    // Modify index register and skip on result == 0.
    //
    // - Short form: add disp to XR[tag] (or to a memory cell if tag
    //   is 0; the historical 1130 has a "modify cell" mode but we
    //   skip that here).
    // - Long form: set XR[tag] to address.
    //
    // After modification, if the resulting value is zero, skip the
    // next instruction.
    let new_value = if shape.is_long() {
        let v = if indirect {
            mem.read_word(address)
        } else {
            address
        };
        if tag != 0 {
            state.write_xr(tag, v);
        }
        v
    } else {
        let prev = state.read_xr(tag);
        let v = prev.wrapping_add(shape.short_disp() as i16 as u16);
        if tag != 0 {
            state.write_xr(tag, v);
        }
        v
    };
    if new_value == 0
        && let Ok((_, size)) = mem.decode_at(state.iar)
    {
        state.iar = state.iar.wrapping_add(size);
    }
    Ok(())
}

/// Area codes for our minimal device subsystem. The historical 1130
/// FC manual assigns specific values per device; these are
/// pedagogical choices for the bring-up demos and may shift when a
/// real I/O subsystem lands.
pub mod area {
    /// 1054 / console Selectric printer (the operator typewriter
    /// integrated into the 1131 CPU). Area code 1 in this emulator.
    pub const CONSOLE: u8 = 1;
}

/// Function codes embedded in the IOCC's second word, top 3 bits of
/// byte 0 (positions 5..7 of the 16-bit big-endian word).
pub mod func {
    pub const WRITE: u8 = 0;
    pub const _READ: u8 = 1;
    pub const _SENSE_INTERRUPT: u8 = 2;
    pub const _CONTROL: u8 = 3;
    pub const _SENSE_DEVICE: u8 = 4;
    pub const _INIT_WRITE: u8 = 5;
    pub const _INIT_READ: u8 = 6;
}

/// XIO: Execute Input/Output.
///
/// The address operand points to a 2-word IOCC (I/O Control
/// Command):
///
/// ```text
/// IOCC + 0:  data address (or immediate, depending on function)
/// IOCC + 1:  high byte = (area_code << 3) | function
///            low  byte = device-specific modifier
/// ```
///
/// For now this emulator handles only `area = CONSOLE`,
/// `function = WRITE`: read one byte from `mem[IOCC+0]` and append
/// it to `state.console_output`. All other (area, function) pairs
/// are silently no-op so codegen / asm output that references XIO
/// in an unhandled way still runs.
fn exec_xio(
    state: &mut CpuState,
    mem: &Memory,
    tag: u8,
    address: u16,
    indirect: bool,
) -> Result<(), ExecError> {
    let iocc_addr = destination_address(state, mem, tag, address, indirect);
    let data_addr = mem.read_word(iocc_addr);
    let control = mem.read_word(iocc_addr.wrapping_add(1));
    let high = (control >> 8) as u8;
    let area = (high >> 3) & 0x1F;
    let function = high & 0x07;
    if area == area::CONSOLE && function == func::WRITE {
        // Take the low byte of the data word as the character to
        // type. (Real 1130 console output is one EBCDIC byte per
        // call; for the bring-up demos we use ASCII literals and
        // postpone the EBCDIC translation to its dedicated saga --
        // see gen-isa/docs/character-encoding-plan.md.)
        let byte = (mem.read_word(data_addr) & 0xFF) as u8;
        state.console_output.push(byte);
    }
    Ok(())
}

fn condition_matches(state: &CpuState, mask: u8) -> bool {
    if mask == 0 {
        return false;
    }
    let acc = state.acc as i16;
    if mask & 0x04 != 0 && (acc & 1) == 0 {
        return true; // E
    }
    if mask & 0x08 != 0 && acc > 0 {
        return true; // +
    }
    if mask & 0x10 != 0 && acc < 0 {
        return true; // -
    }
    if mask & 0x20 != 0 && acc == 0 {
        return true; // Z
    }
    if mask & 0x40 != 0 && state.carry {
        return true; // C
    }
    false
}
