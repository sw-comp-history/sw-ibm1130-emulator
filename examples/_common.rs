//! Shared scaffolding for the demo runners.
//!
//! Each per-demo example imports this via `#[path = "_common.rs"]
//! mod common;` and calls `run_demo()` to assemble, run, and dump
//! the source + bytes + execution stats. Per-demo prints (the
//! "result" of each program) live in the example file itself.

use sw_ibm1130_asm::{SymbolTable, assemble};
use sw_ibm1130_emulator::{CpuState, Memory, run};

const MAX_STEPS: u64 = 10_000;

pub fn run_demo(name: &str, source: &str) -> (CpuState, Memory, SymbolTable) {
    println!("=== {name} ===");
    println!("--- source ---");
    println!("{source}");
    let asm = assemble(source).expect("assemble");
    let byte_len = asm.bytes.len();
    println!("--- assembled ({byte_len} bytes) ---");
    print_hex_dump(&asm.bytes);
    println!();
    let mut mem = Memory::new(4096);
    mem.load_bytes(0, &asm.bytes);
    let mut state = CpuState::new();
    let steps = run(&mut state, &mut mem, MAX_STEPS).expect("run");
    println!(
        "--- ran {steps} instructions; halted = {} ---",
        state.halted
    );
    let after = read_back(&mem, byte_len);
    println!("--- memory after run (same range; '*' marks bytes changed by exec) ---");
    print_hex_dump_diff(&after, &asm.bytes);
    println!();
    (state, mem, asm.symbols)
}

fn read_back(mem: &Memory, byte_len: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(byte_len);
    let words = byte_len.div_ceil(2);
    for w in 0..words {
        let v = mem.read_word(w as u16);
        out.push((v >> 8) as u8);
        out.push((v & 0xFF) as u8);
    }
    out.truncate(byte_len);
    out
}

pub fn print_hex_dump(bytes: &[u8]) {
    for (i, chunk) in bytes.chunks(16).enumerate() {
        print!("  {:04x}: ", i * 16);
        for b in chunk {
            print!("{b:02x} ");
        }
        for _ in chunk.len()..16 {
            print!("   ");
        }
        print!(" |");
        for b in chunk {
            let c = if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            };
            print!("{c}");
        }
        println!("|");
    }
}

/// Hex dump of `bytes`, marking each byte that differs from
/// `baseline` with a `*` prefix instead of a space. The ASCII gutter
/// shows post-run characters.
pub fn print_hex_dump_diff(bytes: &[u8], baseline: &[u8]) {
    for (i, chunk) in bytes.chunks(16).enumerate() {
        let base_off = i * 16;
        print!("  {base_off:04x}: ");
        for (j, b) in chunk.iter().enumerate() {
            let baseline_byte = baseline.get(base_off + j).copied().unwrap_or(0);
            if *b != baseline_byte {
                print!("*{b:02x}");
            } else {
                print!(" {b:02x}");
            }
        }
        for _ in chunk.len()..16 {
            print!("   ");
        }
        print!(" |");
        for b in chunk {
            let c = if b.is_ascii_graphic() || *b == b' ' {
                *b as char
            } else {
                '.'
            };
            print!("{c}");
        }
        println!("|");
    }
}
