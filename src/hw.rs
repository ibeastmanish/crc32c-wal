//! Hardware-accelerated CRC32C implementations with runtime and compile-time detection.

use crate::table::crc32c_slice8;

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "crc")]
unsafe fn crc32c_aarch64_hw(mut crc: u32, mut data: &[u8]) -> u32 {
    use core::arch::aarch64::{__crc32cb, __crc32cd, __crc32ch, __crc32cw};

    crc = !crc;

    // Align to 8 bytes using byte-level instructions
    while !data.is_empty() && (data.as_ptr() as usize % 8) != 0 {
        crc = __crc32cb(crc, data[0]);
        data = &data[1..];
    }

    // Process 8-byte (64-bit) chunks
    while data.len() >= 8 {
        let chunk = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        crc = __crc32cd(crc, chunk);
        data = &data[8..];
    }

    // Process 4-byte chunk if present
    if data.len() >= 4 {
        let chunk = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        crc = __crc32cw(crc, chunk);
        data = &data[4..];
    }

    // Process 2-byte chunk if present
    if data.len() >= 2 {
        let chunk = u16::from_le_bytes([data[0], data[1]]);
        crc = __crc32ch(crc, chunk);
        data = &data[2..];
    }

    // Process trailing byte if present
    if !data.is_empty() {
        crc = __crc32cb(crc, data[0]);
    }

    !crc
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.2")]
unsafe fn crc32c_x86_64_hw(mut crc: u32, mut data: &[u8]) -> u32 {
    use core::arch::x86_64::{_mm_crc32_u64, _mm_crc32_u8};

    crc = !crc;
    let mut crc64 = crc as u64;

    while !data.is_empty() && (data.as_ptr() as usize % 8) != 0 {
        crc64 = _mm_crc32_u8(crc64 as u32, data[0]) as u64;
        data = &data[1..];
    }

    while data.len() >= 8 {
        let chunk = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        crc64 = _mm_crc32_u64(crc64, chunk);
        data = &data[8..];
    }

    crc = crc64 as u32;
    for &byte in data {
        crc = _mm_crc32_u8(crc, byte);
    }

    !crc
}

/// Dispatches CRC32C computation, selecting hardware acceleration when available.
pub fn crc32c_dispatch(crc: u32, data: &[u8]) -> u32 {
    #[cfg(all(target_arch = "aarch64", target_feature = "crc"))]
    {
        return unsafe { crc32c_aarch64_hw(crc, data) };
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "sse4.2"))]
    {
        return unsafe { crc32c_x86_64_hw(crc, data) };
    }

    #[allow(unreachable_code)]
    crc32c_slice8(crc, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_matches_standard_vector() {
        let input = b"123456789";
        let result = crc32c_dispatch(0, input);
        assert_eq!(result, 0xE306_9283);
    }

    #[test]
    fn test_dispatch_matches_slice8_on_large_buffer() {
        let mut buffer = [0u8; 1024];
        for (i, byte) in buffer.iter_mut().enumerate() {
            *byte = (i * 31 % 256) as u8;
        }

        let slice8_result = crc32c_slice8(0, &buffer);
        let dispatch_result = crc32c_dispatch(0, &buffer);
        assert_eq!(slice8_result, dispatch_result);
    }
}
