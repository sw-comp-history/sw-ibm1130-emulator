//! Demo runner: word-by-word "string" copy.
//!
//! Copies words from SRC to DST until the sentinel 0xFFFF; counts
//! how many words were copied in LEN. Demonstrates LDX, indexed
//! LD/STO via XR1, and MDX. The sample SRC is three arbitrary
//! 16-bit words plus a sentinel; the values 0x48/0x49/0x4A spell
//! "HIJ" if interpreted as ASCII (purely a coincidence; encoding
//! is deferred to a future saga).
//!
//! Run with: `cargo run --example strings`

#[path = "_common.rs"]
mod common;

fn main() {
    let (_state, mem, symbols) =
        common::run_demo("strings", include_str!("../tests/programs/strings.asm"));
    let dst = symbols.lookup("DST").unwrap() as u16;
    let len = mem.read_word(symbols.lookup("LEN").unwrap() as u16);
    println!("--- result ---");
    println!("  LEN (words copied) = {len}");
    print!("  DST words = [");
    for i in 0..4 {
        let w = mem.read_word(dst + i);
        if i > 0 {
            print!(", ");
        }
        print!("0x{w:04x}");
    }
    println!("]");
    print!("  DST as ASCII chars (low byte) = \"");
    for i in 0..len {
        let w = mem.read_word(dst + i);
        let c = (w & 0xFF) as u8;
        if c.is_ascii_graphic() {
            print!("{}", c as char);
        } else {
            print!(".");
        }
    }
    println!("\"");
}
