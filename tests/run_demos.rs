//! End-to-end demos: assemble each `tests/programs/*.asm`, run it
//! to halt on the emulator, and assert on the final memory + register
//! state. These programs double as worked examples illustrating the
//! 1130 programming primitives (math, conditions, loops, strings).

use sw_ibm1130_asm::assemble;
use sw_ibm1130_emulator::{CpuState, Memory, run};

const MAX_STEPS: u64 = 10_000;

/// Assemble a source string, load at word 0, run until WAIT, and
/// return the final state + memory.
fn run_program(src: &str) -> (CpuState, Memory, sw_ibm1130_asm::SymbolTable) {
    let asm = assemble(src).expect("assemble");
    let mut mem = Memory::new(4096);
    mem.load_bytes(0, &asm.bytes);
    let mut state = CpuState::new();
    let _steps = run(&mut state, &mut mem, MAX_STEPS).expect("run");
    assert!(
        state.halted,
        "program did not halt within {MAX_STEPS} steps"
    );
    (state, mem, asm.symbols)
}

#[test]
fn math_demo_computes_5_plus_3_times_4() {
    let src = include_str!("programs/math.asm");
    let (_state, mem, symbols) = run_program(src);
    let result_addr = symbols.lookup("RESULT").unwrap() as u16;
    // STD writes the 32-bit product across two words: RESULT = ACC
    // (high half, = 0 for this small product), RESULT+1 = EXT (low
    // half, = 32).
    let high = mem.read_word(result_addr);
    let low = mem.read_word(result_addr + 1);
    let combined = ((high as u32) << 16) | (low as u32);
    assert_eq!(high, 0, "high word of (5+3)*4 should be 0");
    assert_eq!(low, 32, "low word of (5+3)*4 should be 32");
    assert_eq!(combined, 32);
}

#[test]
fn conditions_demo_picks_max() {
    let src = include_str!("programs/conditions.asm");
    let (_state, mem, symbols) = run_program(src);
    let result_addr = symbols.lookup("RESULT").unwrap() as u16;
    // A=12, B=17 -> max is 17.
    assert_eq!(mem.read_word(result_addr), 17);
}

#[test]
fn loops_demo_sums_1_through_10() {
    let src = include_str!("programs/loops.asm");
    let (_state, mem, symbols) = run_program(src);
    let result_addr = symbols.lookup("RESULT").unwrap() as u16;
    // 1+2+...+10 = 55.
    assert_eq!(mem.read_word(result_addr), 55);
}

#[test]
fn strings_demo_copies_until_sentinel() {
    let src = include_str!("programs/strings.asm");
    let (_state, mem, symbols) = run_program(src);
    let dst_addr = symbols.lookup("DST").unwrap() as u16;
    let len_addr = symbols.lookup("LEN").unwrap() as u16;
    assert_eq!(mem.read_word(dst_addr), 0x0048);
    assert_eq!(mem.read_word(dst_addr + 1), 0x0049);
    assert_eq!(mem.read_word(dst_addr + 2), 0x004A);
    assert_eq!(mem.read_word(len_addr), 3);
}

#[test]
fn hello_world_demo_prints_to_console() {
    let src = include_str!("programs/hello.asm");
    let (state, _mem, _symbols) = run_program(src);
    let printed = String::from_utf8(state.console_output).expect("ASCII");
    assert_eq!(printed, "HELLO, WORLD!\n");
}
