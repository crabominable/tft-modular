//! Stable FNV-1a hashing of match-visible state (no pointer/address entropy).

/// FNV-1a 64-bit offset basis.
pub const FNV_OFFSET: u64 = 0xcbf29ce484222325;
/// FNV-1a 64-bit prime.
pub const FNV_PRIME: u64 = 0x100000001b3;

/// Incremental FNV-1a hasher.
#[derive(Clone, Debug)]
pub struct StateHasher {
    state: u64,
}

impl Default for StateHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl StateHasher {
    pub fn new() -> Self {
        Self { state: FNV_OFFSET }
    }

    pub fn finish(self) -> u64 {
        self.state
    }

    pub fn write_u8(&mut self, v: u8) {
        self.state ^= u64::from(v);
        self.state = self.state.wrapping_mul(FNV_PRIME);
    }

    pub fn write_u32(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.write_u8(b);
        }
    }

    pub fn write_u64(&mut self, v: u64) {
        for b in v.to_le_bytes() {
            self.write_u8(b);
        }
    }

    pub fn write_i32(&mut self, v: i32) {
        self.write_u32(v as u32);
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.write_u8(*b);
        }
    }

    pub fn write_str(&mut self, s: &str) {
        self.write_u32(s.len() as u32);
        self.write_bytes(s.as_bytes());
    }
}

/// Convenience: hash raw bytes with FNV-1a.
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut h = StateHasher::new();
    h.write_bytes(bytes);
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_inputs_same_hash() {
        let mut a = StateHasher::new();
        a.write_str("ember_scout");
        a.write_i32(20);
        a.write_u8(1);
        let mut b = StateHasher::new();
        b.write_str("ember_scout");
        b.write_i32(20);
        b.write_u8(1);
        assert_eq!(a.finish(), b.finish());
    }

    #[test]
    fn different_inputs_diverge() {
        let mut a = StateHasher::new();
        a.write_u8(1);
        let mut b = StateHasher::new();
        b.write_u8(2);
        assert_ne!(a.finish(), b.finish());
    }
}
