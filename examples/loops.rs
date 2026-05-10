//! Demo runner: counted loop.
//!
//! Computes `sum 1..10 = 55` using a manual counter and the 1130's
//! skip+jump conditional-branch idiom.
//!
//! Run with: `cargo run --example loops`

#[path = "_common.rs"]
mod common;

fn main() {
    let (state, mem, symbols) =
        common::run_demo("loops", include_str!("../tests/programs/loops.asm"));
    let sum = mem.read_word(symbols.lookup("RESULT").unwrap() as u16);
    let i = mem.read_word(symbols.lookup("I").unwrap() as u16);
    println!("--- result ---");
    println!("  RESULT (sum 1..10)  = {sum}");
    println!("  I      (counter)    = {i}");
    println!("  instructions executed = {}", state.instr_count);
}
