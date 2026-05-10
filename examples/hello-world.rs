//! End-to-end "hello, world" on the IBM 1130.
//!
//! Assembles `tests/programs/hello.asm`, loads the bytes into a
//! 4096-word emulator, runs to halt, and prints whatever the
//! program typed on the 1054/console Selectric printer.
//!
//! Run with: `cargo run --example hello-world`

use sw_ibm1130_asm::assemble;
use sw_ibm1130_emulator::{CpuState, Memory, run};

const SOURCE: &str = include_str!("../tests/programs/hello.asm");
const MAX_STEPS: u64 = 10_000;

fn main() {
    println!("=== source (tests/programs/hello.asm) ===");
    println!("{SOURCE}");

    let asm = assemble(SOURCE).expect("assemble");
    println!("=== assembled bytes ({} bytes total) ===", asm.bytes.len());
    print_hex_dump(&asm.bytes);
    println!();

    let mut mem = Memory::new(4096);
    mem.load_bytes(0, &asm.bytes);
    let mut state = CpuState::new();
    let steps = run(&mut state, &mut mem, MAX_STEPS).expect("run");
    println!(
        "=== ran {} instructions; halted = {} ===",
        steps, state.halted
    );

    println!("=== captured 1054/console output ===");
    let output = String::from_utf8_lossy(&state.console_output);
    print!("{output}");
    if !output.ends_with('\n') {
        println!();
    }
}

fn print_hex_dump(bytes: &[u8]) {
    for (i, chunk) in bytes.chunks(16).enumerate() {
        print!("  {:04x}: ", i * 16);
        for b in chunk {
            print!("{:02x} ", b);
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
