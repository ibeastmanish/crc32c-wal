//! State machine and Hasher implementation for rolling CRC32C checksums.

use core::hash::Hasher;
use crate::hw::crc32c_dispatch;

/// Rolling state machine for incremental CRC32C computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crc32c {
    state: u32,
}

impl Default for Crc32c {
    fn default() -> Self {
        Self::new()
    }
}

impl Crc32c {
    /// Creates a new CRC32C accumulator initialized to zero.
    pub const fn new() -> Self {
        Self { state: 0 }
    }

    /// Creates a new accumulator with a pre-existing CRC seed.
    pub const fn with_initial(initial: u32) -> Self {
        Self { state: initial }
    }

    /// Feeds a contiguous byte slice into the rolling checksum.
    pub fn update(&mut self, data: &[u8]) {
        self.state = crc32c_dispatch(self.state, data);
    }

    /// Resets the accumulator back to initial zero state.
    pub fn reset(&mut self) {
        self.state = 0;
    }

    /// Returns the finalized 32-bit CRC32C checksum.
    pub fn finalize(&self) -> u32 {
        self.state
    }
}

impl Hasher for Crc32c {
    fn finish(&self) -> u64 {
        self.state as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }
}

/// Computes the CRC32C checksum of a single byte slice.
pub fn crc32c(data: &[u8]) -> u32 {
    crc32c_dispatch(0, data)
}

/// Extends a previously computed CRC32C checksum with additional data.
pub fn crc32c_append(crc: u32, data: &[u8]) -> u32 {
    crc32c_dispatch(crc, data)
}

/// Computes a composite CRC32C checksum across disjoint header and payload buffers without concatenation.
pub fn crc32c_wal_frame(header: &[u8], payload: &[u8]) -> u32 {
    let mut state = crc32c_dispatch(0, header);
    state = crc32c_dispatch(state, payload);
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incremental_equals_monolithic() {
        let part1 = b"1234";
        let part2 = b"56789";

        let mut hasher = Crc32c::new();
        hasher.update(part1);
        hasher.update(part2);

        assert_eq!(hasher.finalize(), 0xE306_9283);
        assert_eq!(crc32c(b"123456789"), 0xE306_9283);
    }

    #[test]
    fn test_wal_frame_segmented() {
        let header = b"WAL_HEADER_BLOCK_001";
        let payload = b"PAYLOAD_BODY_BYTES_HERE";

        let mut combined = Vec::new();
        combined.extend_from_slice(header);
        combined.extend_from_slice(payload);

        let segmented = crc32c_wal_frame(header, payload);
        let monolithic = crc32c(&combined);

        assert_eq!(segmented, monolithic);
    }
}
