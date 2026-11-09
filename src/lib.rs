//! # crc32c-wal
//!
//! Hardware-accelerated CRC32C (Castagnoli) checksum utility optimized for write-ahead log (WAL)
//! record validation, database persistence engines, and storage block integrity.
//!
//! ## Highlights
//!
//! - **Hardware Acceleration:** Employs ARMv8-A CRC instructions on AArch64 and SSE 4.2 on x86_64.
//! - **Slice-by-8 Fallback:** Processes 8 bytes per cycle simultaneously in pure software (>1.5 GB/s).
//! - **Segmented WAL Validation:** Computes checksums across disjoint header and payload buffers without memory concatenation.
//! - **`no_std` Native:** Usable on bare-metal firmware, embedded kernels, and WASM runtimes.
//! - **Zero Runtime Dependencies:** Self-contained and ultra-lightweight.
//!
//! ## Quick Start
//!
//! ```rust
//! use crc32c_wal::{crc32c, crc32c_append, crc32c_wal_frame};
//!
//! // 1. Compute checksum of a single block
//! let checksum = crc32c(b"123456789");
//! assert_eq!(checksum, 0xE306_9283);
//!
//! // 2. Segmented check across WAL header and body without copying
//! let header = b"WAL_TX_HEADER";
//! let body = b"KEY_VALUE_MUTATION_DATA";
//! let frame_checksum = crc32c_wal_frame(header, body);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub mod hasher;
pub mod hw;
pub mod table;

pub use hasher::{crc32c, crc32c_append, crc32c_wal_frame, Crc32c};
pub use hw::crc32c_dispatch;
pub use table::{crc32c_slice8, CASTAGNOLI_POLY, SLICE8_TABLES};
