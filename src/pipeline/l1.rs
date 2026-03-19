//! # L1 — Cleaned Records
//!
//! The L1 layer takes raw bytes from L0 and parses them into structured,
//! typed records. This is a purely deterministic transformation — no ML,
//! no API calls, no fuzzy logic.
//!
//! ## Inputs
//!
//! - **L0 artifacts**: Raw downloaded files (CSV, TSV, XML, shapefiles)
//!   along with their acquisition manifests.
//!
//! ## Processing
//!
//! - Parse each source format according to its documented schema.
//! - Normalize whitespace, fix encoding issues (e.g. Latin-1 → UTF-8).
//! - Split combined name fields into components (first, middle, last,
//!   suffix, nickname) without discarding any signal.
//! - Normalize party labels to a controlled vocabulary while preserving
//!   the original string.
//! - Convert vote counts to integers; flag and quarantine unparseable rows.
//! - Assign deterministic record IDs derived from a hash of (source, row).
//!
//! ## Outputs
//!
//! - **`L1Record`**: A structured record with all name components, vote
//!   totals, geographic identifiers, and a hash-chain link back to L0.
//! - **Quarantine log**: Rows that failed parsing, with error details.
//!
//! ## Guarantees
//!
//! - Deterministic: identical L0 input always produces identical L1 output.
//! - Lossless: every field from the source is preserved, even if not yet
//!   mapped to a canonical schema column.

/// A single cleaned election record produced by the L1 layer.
///
/// Contains parsed, structured fields derived from a raw source row.
/// Every `L1Record` carries a hash linking it back to the originating
/// L0 artifact and byte offset.
pub struct L1Record;
