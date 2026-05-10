//! Instruction execution: one `step()` per instruction.
//!
//! BSC condition mask semantics (chosen for this emulator; pinned
//! here authoritatively per saga step 11):
//!
//! ```text
//! mask bit | meaning when set
//! ---------+-----------------------------------------------------
//! 0x01     | Z   -- ACC == 0
//! 0x02     | -   -- ACC < 0 (two's-complement)
//! 0x04     | +   -- ACC > 0
//! 0x08     | E   -- ACC even (low bit clear)
//! 0x10     | C   -- carry indicator set
//! 0x20     | O   -- overflow indicator set
//! ```
//!
//! **BSC short** (`BSC mask`): tests the displacement byte as the
//! mask. If any masked condition matches, **skip** the next
//! instruction (advance IAR by that instruction's word size).
//! Mask = 0 means no condition tested -> never skip.
//!
//! **BSC long** (`BSC L target [, mask]`): always-unconditional
//! branch to `target` in this emulator. The historical 1130 also
//! supported a mask in the long form's reserved bits, but the ISA
//! spec at saga step 7 marked those bits as reserved-zero; the
//! `mask` operand the asm accepts is parsed but currently dropped
//! (acknowledged limitation, postmortem item for step 12).
//!
//! Conditional branch idiom on this emulator: `BSC mask` (skip on
//! condition) followed by `BSC L target, 0` (unconditional jump).
//! The combination effects "branch unless condition matched".
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
            address,
        } => exec_long(state, mem, op, tag, indirect, address),
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
    address: u16,
) -> Result<(), ExecError> {
    exec_with_address(state, mem, op, tag, indirect, address, LongShape)
}

#[derive(Copy, Clone)]
struct LongShape;

trait FormShape: Copy {
    fn is_long(self) -> bool;
    fn short_disp(self) -> i8;
}

impl FormShape for ShortShape {
    fn is_long(self) -> bool {
        false
    }
    fn short_disp(self) -> i8 {
        self.disp
    }
}

impl FormShape for LongShape {
    fn is_long(self) -> bool {
        true
    }
    fn short_disp(self) -> i8 {
        0
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
            // BSI: write IAR to target, branch to target+1.
            let target = destination_address(state, mem, tag, address, indirect);
            mem.write_word(target, state.iar);
            state.iar = target.wrapping_add(1);
            Ok(())
        }
        Opcode::ModifyIndex => exec_mdx(state, mem, tag, indirect, address, shape),
        Opcode::Wait => {
            state.halted = true;
            Ok(())
        }
        Opcode::ExecuteInputOutput => {
            // I/O is out of scope for step 11; treat as no-op so
            // programs containing it don't crash. A real impl
            // would dispatch to a device subsystem.
            Ok(())
        }
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
        // Long form: unconditional branch in this emulator (mask
        // bits in the historical first-word reserved area aren't
        // exposed by our ISA spec; documented in module-level docs
        // and as a step-12 postmortem item).
        let target = destination_address(state, mem, tag, address, indirect);
        state.iar = target;
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

fn condition_matches(state: &CpuState, mask: u8) -> bool {
    if mask == 0 {
        return false;
    }
    let acc = state.acc as i16;
    if mask & 0x01 != 0 && acc == 0 {
        return true;
    }
    if mask & 0x02 != 0 && acc < 0 {
        return true;
    }
    if mask & 0x04 != 0 && acc > 0 {
        return true;
    }
    if mask & 0x08 != 0 && (acc & 1) == 0 {
        return true;
    }
    if mask & 0x10 != 0 && state.carry {
        return true;
    }
    if mask & 0x20 != 0 && state.overflow {
        return true;
    }
    false
}
