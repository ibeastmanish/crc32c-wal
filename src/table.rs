//! Slice-by-8 lookup table and software engine for CRC32C (Castagnoli).

pub const CASTAGNOLI_POLY: u32 = 0x82F6_3B78;

/// Generates the standard 256-entry single-byte Castagnoli lookup table.
const fn generate_table_0() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ CASTAGNOLI_POLY;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

/// Generates the 8 x 256 slice-by-8 matrix at compile time.
const fn generate_slice8_tables() -> [[u32; 256]; 8] {
    let mut tables = [[0u32; 256]; 8];
    tables[0] = generate_table_0();

    let mut t = 1;
    while t < 8 {
        let mut i = 0;
        while i < 256 {
            let prev = tables[t - 1][i];
            let index = (prev & 0xFF) as usize;
            tables[t][i] = (prev >> 8) ^ tables[0][index];
            i += 1;
        }
        t += 1;
    }
    tables
}

pub static SLICE8_TABLES: [[u32; 256]; 8] = generate_slice8_tables();

/// Computes CRC32C using the slice-by-8 parallel lookup algorithm.
pub fn crc32c_slice8(mut crc: u32, mut data: &[u8]) -> u32 {
    crc = !crc;

    // Process leading unaligned bytes until 8-byte aligned
    while !data.is_empty() && (data.as_ptr() as usize % 8) != 0 {
        let index = ((crc ^ (data[0] as u32)) & 0xFF) as usize;
        crc = (crc >> 8) ^ SLICE8_TABLES[0][index];
        data = &data[1..];
    }

    // Process 8 bytes simultaneously per iteration
    while data.len() >= 8 {
        let b0 = (crc as u8) ^ data[0];
        let b1 = ((crc >> 8) as u8) ^ data[1];
        let b2 = ((crc >> 16) as u8) ^ data[2];
        let b3 = ((crc >> 24) as u8) ^ data[3];
        let b4 = data[4];
        let b5 = data[5];
        let b6 = data[6];
        let b7 = data[7];

        crc = SLICE8_TABLES[7][b0 as usize]
            ^ SLICE8_TABLES[6][b1 as usize]
            ^ SLICE8_TABLES[5][b2 as usize]
            ^ SLICE8_TABLES[4][b3 as usize]
            ^ SLICE8_TABLES[3][b4 as usize]
            ^ SLICE8_TABLES[2][b5 as usize]
            ^ SLICE8_TABLES[1][b6 as usize]
            ^ SLICE8_TABLES[0][b7 as usize];

        data = &data[8..];
    }

    // Process remaining trailing bytes
    for &byte in data {
        let index = ((crc ^ (byte as u32)) & 0xFF) as usize;
        crc = (crc >> 8) ^ SLICE8_TABLES[0][index];
    }

    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_castagnoli_test_vector() {
        // Standard Castagnoli 32C test vector:
        // ASCII b"123456789" => 0xE3069283
        let input = b"123456789";
        let result = crc32c_slice8(0, input);
        assert_eq!(result, 0xE306_9283);
    }

    #[test]
    fn test_empty_slice() {
        assert_eq!(crc32c_slice8(0, b""), 0);
    }

    #[test]
    fn test_various_lengths() {
        let data = [0x55u8; 127];
        let res1 = crc32c_slice8(0, &data);
        assert_ne!(res1, 0);

        // Deterministic repeat
        let res2 = crc32c_slice8(0, &data);
        assert_eq!(res1, res2);
    }
}
