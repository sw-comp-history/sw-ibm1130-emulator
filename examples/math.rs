//! Demo runner: arithmetic.
//!
//! `(5 + 3) * 4 = 32`. The 1130's `M` (multiply) puts the 32-bit
//! product in the (ACC, EXT) pair, so for a small product the high
//! half (ACC, then stored to RESULT) is zero and the actual answer
//! is in EXT.
//!
//! Run with: `cargo run --example math`

#[path = "_common.rs"]
mod common;

fn main() {
    let (state, mem, symbols) =
        common::run_demo("math", include_str!("../tests/programs/math.asm"));
    let result_addr = symbols.lookup("RESULT").unwrap() as u16;
    println!("--- result ---");
    println!(
        "  RESULT (high half of multiply pair) = {}",
        mem.read_word(result_addr)
    );
    println!("  EXT    (low  half of multiply pair) = {}", state.ext);
    println!("  ACC                                 = {}", state.acc);
    println!(
        "  -> answer = {} (low half holds the small product)",
        state.ext
    );
}
