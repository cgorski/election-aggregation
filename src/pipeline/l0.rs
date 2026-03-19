//! # L0 — Raw Acquisition Layer
//!
//! The L0 layer stores byte-identical copies of source files exactly as
//! downloaded, along with acquisition metadata (manifests).
//!
//! ## Inputs
//!
//! - Remote URLs or local file paths pointing to election data files
//!   (CSV, TSV, XML, shapefiles, ZIP archives, etc.)
//!
//! ## Outputs
//!
//! - **Raw bytes**: The source file stored verbatim, never modified.
//! - **Acquisition manifest**: A sidecar record capturing:
//!   - Source URL / file path
//!   - Download timestamp (UTC)
//!   - SHA-256 hash of the raw bytes
//!   - HTTP headers / ETag (when available)
//!   - File size in bytes
//!
//! ## Invariants
//!
//! - L0 is **append-only**: once a file is stored, it is never overwritten.
//! - Re-downloading the same URL produces a new versioned entry if the
//!   content hash differs from the previous acquisition.
//! - No parsing, cleaning, or transformation occurs at this layer.
//! - The SHA-256 hash forms the root of the hash chain that subsequent
//!   layers extend.

/// Placeholder for a raw L0 acquisition record.
///
/// Will eventually hold the raw bytes (or a path to them) plus the
/// acquisition manifest metadata.
pub struct L0Record;
