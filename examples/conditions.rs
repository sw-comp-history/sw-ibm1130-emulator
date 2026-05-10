//! Demo runner: conditional branching.
//!
//! Computes `max(A, B)` using the standard 1130 idiom:
//! `BSC mask` (skip on condition) then `BSC L target, 0`
//! (unconditional jump). With A=12 and B=17, RESULT should be 17.
//!
//! Run with: `cargo run --example conditions`

#[path = "_common.rs"]
mod common;

fn main() {
    let (_state, mem, symbols) = common::run_demo(
        "conditions",
        include_str!("../tests/programs/conditions.asm"),
    );
    let a = mem.read_word(symbols.lookup("A").unwrap() as u16);
    let b = mem.read_word(symbols.lookup("B").unwrap() as u16);
    let r = mem.read_word(symbols.lookup("RESULT").unwrap() as u16);
    println!("--- result ---");
    println!("  A      = {a}");
    println!("  B      = {b}");
    println!("  RESULT = max(A, B) = {r}");
}
