//! Word-addressed memory.
//!
//! 1130 memory is word-addressed: address N refers to a 16-bit word.
//! Default size is 4096 words (8 KB), the smallest configuration in
//! the historical lineup; the constructor accepts any size.
//!
//! Bytes interfaces are provided for the decoder, which expects
//! big-endian byte streams. Word N occupies byte addresses (N*2,
//! N*2+1).

use sw_isa_core::DecodeError;

/// Word-addressed memory. Reads/writes treat each address as a
/// 16-bit word; conversions to/from the byte stream use big-endian.
#[derive(Debug, Clone)]
pub struct Memory {
    words: Vec<u16>,
}

impl Memory {
    /// Allocate `size_words` words, all zero. The 1130 minimum was
    /// 4096 words; tests can pick any size.
    pub fn new(size_words: usize) -> Self {
        Self {
            words: vec![0; size_words],
        }
    }

    pub fn size_words(&self) -> usize {
        self.words.len()
    }

    pub fn read_word(&self, addr: u16) -> u16 {
        self.words.get(addr as usize).copied().unwrap_or(0)
    }

    pub fn write_word(&mut self, addr: u16, value: u16) {
        if let Some(slot) = self.words.get_mut(addr as usize) {
            *slot = value;
        }
    }

    /// Load a contiguous program at the given word address. Each
    /// pair of bytes becomes one big-endian word.
    pub fn load_bytes(&mut self, word_addr: u16, bytes: &[u8]) {
        let mut idx = word_addr as usize;
        let mut i = 0;
        while i + 1 < bytes.len() && idx < self.words.len() {
            self.words[idx] = u16::from_be_bytes([bytes[i], bytes[i + 1]]);
            i += 2;
            idx += 1;
        }
        // Trailing odd byte (if any) is zero-extended into the high
        // half of the next word.
        if i < bytes.len() && idx < self.words.len() {
            self.words[idx] = (bytes[i] as u16) << 8;
        }
    }

    /// Read up to 4 bytes starting at the byte address corresponding
    /// to `word_addr` (big-endian). Used by the decoder.
    pub fn fetch_bytes(&self, word_addr: u16) -> [u8; 4] {
        let w0 = self.read_word(word_addr).to_be_bytes();
        let w1 = self.read_word(word_addr.wrapping_add(1)).to_be_bytes();
        [w0[0], w0[1], w1[0], w1[1]]
    }

    /// Decode the instruction at `word_addr` and return it plus its
    /// size in *words* (1 for short, 2 for long).
    pub fn decode_at(
        &self,
        word_addr: u16,
    ) -> Result<(sw_ibm1130_isa::Instruction, u16), DecodeError> {
        let bytes = self.fetch_bytes(word_addr);
        let (insn, n) = sw_ibm1130_isa::decode::decode(&bytes)?;
        Ok((insn, (n / 2) as u16))
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new(4096)
    }
}
