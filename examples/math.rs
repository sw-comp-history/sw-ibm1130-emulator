//! Demo runner: arithmetic.
//!
//! `(5 + 3) * 4 = 32`. The 1130's `M` (multiply) puts the 32-bit
//! product in the (ACC, EXT) pair; `STD` (store double) writes the
//! pair to consecutive words, so RESULT and RESULT+1 together hold
//! the full answer.
//!
//! Run with: `cargo run --example math`

#[path = "_common.rs"]
mod common;

fn main() {
    let (_state, mem, symbols) =
        common::run_demo("math", include_str!("../tests/programs/math.asm"));
    let result_addr = symbols.lookup("RESULT").unwrap() as u16;
    let high = mem.read_word(result_addr);
    let low = mem.read_word(result_addr + 1);
    let combined = ((high as u32) << 16) | (low as u32);
    println!("--- result ---");
    println!("  RESULT     (high word) = {high}");
    println!("  RESULT+1   (low  word) = {low}");
    println!("  combined 32-bit answer = {combined}    (= (5 + 3) * 4)");
}
