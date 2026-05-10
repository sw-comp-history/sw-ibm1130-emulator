//! `sw-ibm1130-emulator`: IBM 1130 instruction execution.
//!
//! Minimal emulator covering the instruction subset that
//! `sw-ibm1130-codegen` produces. See `src/exec.rs` for the BSC
//! mask semantics that this crate pins authoritatively.

pub mod exec;
pub mod memory;
pub mod state;

pub use exec::{ExecError, run, step};
pub use memory::Memory;
pub use state::CpuState;

use sw_isa_core::DecodeError;

/// Convenience: assemble an `&str` of source via `sw-ibm1130-asm`,
/// load it at word address 0, and return the prepared CPU + memory.
/// Lives behind a `cfg(any(test, feature = ...))` guard in production
/// usage; here we expose it always for simplicity since the dev-dep
/// chain is small.
///
/// Note: `sw-ibm1130-asm` is a dev-dep, so this helper lives in
/// `tests/helpers.rs` rather than the library proper.
#[doc(hidden)]
pub fn run_until_halt(
    state: &mut CpuState,
    mem: &mut Memory,
    max_steps: u64,
) -> Result<u64, ExecError> {
    run(state, mem, max_steps)
}

/// Re-exported for callers that want to handle decode errors
/// directly without pulling in `sw-isa-core`.
pub type DecodeErr = DecodeError;
