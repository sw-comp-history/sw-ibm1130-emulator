//! End-to-end "hello, world" on the IBM 1130.
//!
//! Run with: `cargo run --example hello-world`

#[path = "_common.rs"]
mod common;

fn main() {
    let (state, _mem, _symbols) = common::run_demo(
        "hello-world (1054/console Selectric)",
        include_str!("../tests/programs/hello.asm"),
    );
    println!("--- captured 1054/console output ---");
    let output = String::from_utf8_lossy(&state.console_output);
    print!("{output}");
    if !output.ends_with('\n') {
        println!();
    }
}
