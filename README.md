# crc32c-wal

Hardware-accelerated CRC32C (Castagnoli) checksum utility optimized for write-ahead log (WAL) record validation, database persistence engines, and storage block integrity.

[![Crates.io](https://img.shields.io/crates/v/crc32c-wal.svg)](https://crates.io/crates/crc32c-wal)
[![Documentation](https://docs.rs/crc32c-wal/badge.svg)](https://docs.rs/crc32c-wal)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

---

## Overview

In write-ahead logging (WAL) and storage engines (such as RocksDB, SQLite WAL, and PostgreSQL), CRC32C (polynomial `0x1EDC6F41`, reversed `0x82F63B78`) is the industry standard checksum. Compared to IEEE 802.3, the Castagnoli polynomial provides significantly superior error detection characteristics for data blocks up to 4KB (page cache and sector boundaries).

`crc32c-wal` is engineered for extreme throughput:
* On **ARMv8-A / Apple Silicon**, it dispatches dedicated hardware instructions (`__crc32cb`, `__crc32cd`).
* On **x86_64**, it executes SSE 4.2 hardware instructions (`_mm_crc32_u64`).
* On embedded architectures without hardware extensions, it executes a high-speed **slice-by-8 parallel table lookup** processing 8 bytes per iteration in software (>1.5 GB/s).

Part of the challenged-networking and crash-resilient storage suite alongside [`dtn-wal`](https://crates.io/crates/dtn-wal), [`crash-sim`](https://crates.io/crates/crash-sim), and [`dtn-proto`](https://crates.io/crates/dtn-proto).

---

## Architecture

```
+--------------------------------------------------------------+
|                     Log Record or Disk Block                 |
+--------------------------------------------------------------+
                               |
                               v
               Hardware Instruction Detection
                               |
              +----------------+----------------+
              |                                 |
              v (Has CRC / SSE4.2)              v (Generic / WASM)
   +---------------------+             +---------------------+
   | Hardware SIMD Path  |             | Slice-by-8 Matrix   |
   | ARMv8-A: __crc32cd  |             | (8 Bytes / Cycle)   |
   | x86_64: _mm_crc32   |             | 8x256 Lookup Tables |
   +---------------------+             +---------------------+
              |                                 |
              +----------------+----------------+
                               |
                               v
             Finalized 32-Bit Castagnoli Checksum
```

---

## Features

* **Hardware Accelerated:** Seamlessly leverages CPU hardware instructions for throughput reaching memory bandwidth limits.
* **Slice-by-8 Software Fallback:** High-performance table-driven fallback without performance cliffs on older targets.
* **Segmented WAL Calculation:** Calculates continuous checksums across disjoint headers and payloads (`crc32c_wal_frame`) without concatenating memory buffers.
* **`no_std` Native:** Suitable for embedded bare-metal kernels, bootloaders, and WebAssembly.
* **Zero Runtime Dependencies:** Completely self-contained.

---

## Installation

Add `crc32c-wal` to your `Cargo.toml`:

```toml
[dependencies]
crc32c-wal = "0.1.0"
```

For bare-metal `no_std` environments:

```toml
[dependencies]
crc32c-wal = { version = "0.1.0", default-features = false }
```

---

## Usage Examples

### 1. Simple Checksum Computation

```rust
use crc32c_wal::crc32c;

fn main() {
    let data = b"123456789";
    let checksum = crc32c(data);
    assert_eq!(checksum, 0xE306_9283);
    println!("CRC32C: 0x{:08X}", checksum);
}
```

### 2. Segmented WAL Frame Validation (Zero Copy)

When appending records to a write-ahead log, the header (transaction ID, timestamp, length) and body (payload) often reside in separate memory slices. `crc32c_wal_frame` computes the composite checksum without allocation:

```rust
use crc32c_wal::crc32c_wal_frame;

fn main() {
    let header = b"LOG_RECORD_HDR_V1";
    let payload = b"DIRTY_PAGE_DATA_SECTOR_4096";

    let checksum = crc32c_wal_frame(header, payload);
    println!("Frame checksum: 0x{:08X}", checksum);
}
```

### 3. Incremental Rolling Checksum

```rust
use crc32c_wal::Crc32c;

fn main() {
    let mut hasher = Crc32c::new();
    hasher.update(b"CHUNK_1_");
    hasher.update(b"CHUNK_2_");
    hasher.update(b"CHUNK_3");

    let final_checksum = hasher.finalize();
    println!("Streaming checksum: 0x{:08X}", final_checksum);
}
```

---

## Ecosystem

* [`dtn-wal`](https://crates.io/crates/dtn-wal): Crash-resilient SQLite WAL queue for Delay-Tolerant Networks.
* [`crash-sim`](https://crates.io/crates/crash-sim): Chaos-testing harness for disk fault injection and torn writes.
* [`dtn-proto`](https://crates.io/crates/dtn-proto): Zero-copy packet serializer and parser for challenged mesh links.

---

## License

Dual-licensed under either:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
* MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
