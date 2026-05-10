//! Unit tests for individual instruction semantics.
//!
//! Tests poke instructions directly into memory (not via the asm)
//! so the emulator's behaviour is pinned independent of the asm's
//! correctness.

use sw_ibm1130_emulator::{CpuState, Memory, step};
use sw_ibm1130_isa::{Instruction, Opcode, encode};

fn write_insn(mem: &mut Memory, addr: u16, insn: Instruction) -> u16 {
    let mut buf = [0u8; 4];
    let n = encode::encode(&insn, &mut buf).unwrap();
    let mut a = addr;
    let mut i = 0;
    while i + 1 < n {
        let w = u16::from_be_bytes([buf[i], buf[i + 1]]);
        mem.write_word(a, w);
        a += 1;
        i += 2;
    }
    a
}

#[test]
fn load_long_form_reads_from_memory() {
    let mut mem = Memory::new(64);
    mem.write_word(20, 0xCAFE);
    write_insn(
        &mut mem,
        0,
        Instruction::Long {
            op: Opcode::Load,
            tag: 0,
            indirect: false,
            address: 20,
        },
    );
    let mut state = CpuState::new();
    step(&mut state, &mut mem).unwrap();
    assert_eq!(state.acc, 0xCAFE);
}

#[test]
fn add_long_form_accumulates() {
    let mut mem = Memory::new(64);
    mem.write_word(20, 7);
    write_insn(
        &mut mem,
        0,
        Instruction::Long {
            op: Opcode::Add,
            tag: 0,
            indirect: false,
            address: 20,
        },
    );
    let mut state = CpuState::new();
    state.acc = 5;
    step(&mut state, &mut mem).unwrap();
    assert_eq!(state.acc, 12);
}

#[test]
fn store_long_form_writes_acc() {
    let mut mem = Memory::new(64);
    write_insn(
        &mut mem,
        0,
        Instruction::Long {
            op: Opcode::Store,
            tag: 0,
            indirect: false,
            address: 30,
        },
    );
    let mut state = CpuState::new();
    state.acc = 0x1234;
    step(&mut state, &mut mem).unwrap();
    assert_eq!(mem.read_word(30), 0x1234);
}

#[test]
fn multiply_writes_pair_to_acc_ext() {
    let mut mem = Memory::new(64);
    mem.write_word(20, 4);
    write_insn(
        &mut mem,
        0,
        Instruction::Long {
            op: Opcode::Multiply,
            tag: 0,
            indirect: false,
            address: 20,
        },
    );
    let mut state = CpuState::new();
    state.acc = 8;
    step(&mut state, &mut mem).unwrap();
    // 8 * 4 = 32 -> ACC (high) = 0, EXT (low) = 32.
    assert_eq!(state.acc, 0);
    assert_eq!(state.ext, 32);
}

#[test]
fn divide_yields_quotient_and_remainder() {
    let mut mem = Memory::new(64);
    mem.write_word(20, 3);
    write_insn(
        &mut mem,
        0,
        Instruction::Long {
            op: Opcode::Divide,
            tag: 0,
            indirect: false,
            address: 20,
        },
    );
    let mut state = CpuState::new();
    // Dividend pair = 17 (high=0, low=17). 17 / 3 = 5 rem 2.
    state.acc = 0;
    state.ext = 17;
    step(&mut state, &mut mem).unwrap();
    assert_eq!(state.acc, 5);
    assert_eq!(state.ext, 2);
}

#[test]
fn divide_by_zero_errors() {
    let mut mem = Memory::new(64);
    mem.write_word(20, 0);
    write_insn(
        &mut mem,
        0,
        Instruction::Long {
            op: Opcode::Divide,
            tag: 0,
            indirect: false,
            address: 20,
        },
    );
    let mut state = CpuState::new();
    state.ext = 10;
    let err = step(&mut state, &mut mem).unwrap_err();
    assert_eq!(err, sw_ibm1130_emulator::ExecError::DivideByZero);
}

#[test]
fn wait_halts() {
    let mut mem = Memory::new(8);
    write_insn(
        &mut mem,
        0,
        Instruction::Short {
            op: Opcode::Wait,
            tag: 0,
            disp: 0,
        },
    );
    let mut state = CpuState::new();
    step(&mut state, &mut mem).unwrap();
    assert!(state.halted);
    assert!(step(&mut state, &mut mem).is_err());
}

#[test]
fn bsi_writes_iar_and_jumps_to_target_plus_one() {
    let mut mem = Memory::new(64);
    write_insn(
        &mut mem,
        0,
        Instruction::Long {
            op: Opcode::BranchStore,
            tag: 0,
            indirect: false,
            address: 30,
        },
    );
    let mut state = CpuState::new();
    step(&mut state, &mut mem).unwrap();
    // BSI long is 2 words, so IAR was 2 after fetch; target=30 gets
    // the return address; IAR should now be 31.
    assert_eq!(mem.read_word(30), 2);
    assert_eq!(state.iar, 31);
}

#[test]
fn bsc_long_unconditional_branches_when_mask_zero() {
    let mut mem = Memory::new(64);
    write_insn(
        &mut mem,
        0,
        Instruction::Long {
            op: Opcode::BranchSkipCondition,
            tag: 0,
            indirect: false,
            address: 50,
        },
    );
    let mut state = CpuState::new();
    state.acc = 0; // doesn't matter; mask is 0
    step(&mut state, &mut mem).unwrap();
    assert_eq!(state.iar, 50);
}

#[test]
fn bsc_short_skips_when_z_matches_acc_zero() {
    // Conditional branch idiom on this emulator:
    //   BSC mask           ; skip next if condition matches
    //   BSC L target, 0    ; unconditional jump
    let mut mem = Memory::new(64);
    write_insn(
        &mut mem,
        0,
        Instruction::Short {
            op: Opcode::BranchSkipCondition,
            tag: 0,
            disp: 0x01, // Z mask
        },
    );
    write_insn(
        &mut mem,
        1,
        Instruction::Long {
            op: Opcode::BranchSkipCondition,
            tag: 0,
            indirect: false,
            address: 50,
        },
    );
    let mut state = CpuState::new();
    state.acc = 0; // Z matches -> skip the next (long) instruction.
    step(&mut state, &mut mem).unwrap();
    // After the short BSC, IAR should be past the long BSC: 1 (size
    // of short) + 2 (size of skipped long) = 3.
    assert_eq!(state.iar, 3);
}

#[test]
fn bsc_short_falls_through_when_condition_does_not_match() {
    let mut mem = Memory::new(64);
    write_insn(
        &mut mem,
        0,
        Instruction::Short {
            op: Opcode::BranchSkipCondition,
            tag: 0,
            disp: 0x01, // Z mask
        },
    );
    write_insn(
        &mut mem,
        1,
        Instruction::Long {
            op: Opcode::BranchSkipCondition,
            tag: 0,
            indirect: false,
            address: 50,
        },
    );
    let mut state = CpuState::new();
    state.acc = 5; // Z does not match -> don't skip.
    step(&mut state, &mut mem).unwrap();
    assert_eq!(state.iar, 1);
    step(&mut state, &mut mem).unwrap();
    assert_eq!(state.iar, 50);
}

#[test]
fn shift_left_by_3() {
    let mut mem = Memory::new(8);
    write_insn(
        &mut mem,
        0,
        Instruction::Short {
            op: Opcode::ShiftLeft,
            tag: 0,
            disp: 3,
        },
    );
    let mut state = CpuState::new();
    state.acc = 0x0007;
    step(&mut state, &mut mem).unwrap();
    assert_eq!(state.acc, 0x0038);
}

#[test]
fn shift_right_arithmetic_preserves_sign() {
    let mut mem = Memory::new(8);
    write_insn(
        &mut mem,
        0,
        Instruction::Short {
            op: Opcode::ShiftRight,
            tag: 0,
            disp: 4,
        },
    );
    let mut state = CpuState::new();
    state.acc = 0x8000_u16; // i16::MIN
    step(&mut state, &mut mem).unwrap();
    // -32768 >> 4 = -2048 = 0xF800
    assert_eq!(state.acc, 0xF800);
}

#[test]
fn and_or_xor() {
    for (op, lhs, rhs, expected) in [
        (Opcode::And, 0xF0F0u16, 0x0FFFu16, 0x00F0u16),
        (Opcode::Or, 0x00F0, 0x0F00, 0x0FF0),
        (Opcode::ExclusiveOr, 0xFFFF, 0x00FF, 0xFF00),
    ] {
        let mut mem = Memory::new(64);
        mem.write_word(20, rhs);
        write_insn(
            &mut mem,
            0,
            Instruction::Long {
                op,
                tag: 0,
                indirect: false,
                address: 20,
            },
        );
        let mut state = CpuState::new();
        state.acc = lhs;
        step(&mut state, &mut mem).unwrap();
        assert_eq!(state.acc, expected, "op {:?}", op);
    }
}
