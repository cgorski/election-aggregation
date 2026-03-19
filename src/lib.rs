//! # Election Aggregation
//!
//! A multi-layer pipeline for collecting, normalizing, and unifying
//! US local election results from heterogeneous sources.
//!
//! ## Architecture
//!
//! The pipeline processes election data through five immutable layers:
//!
//! - **L0 (Raw)**: Byte-identical source files with acquisition manifests
//! - **L1 (Cleaned)**: Parsed, structured records with all name components preserved
//! - **L2 (Embedded)**: Vector embeddings for fuzzy matching and classification
//! - **L3 (Matched)**: Entity-resolved records with candidate and contest identifiers
//! - **L4 (Canonical)**: Authoritative names, temporal chains, and verification
//!
//! Each layer depends on all prior layers. Every record carries a hash chain
//! back to the original source bytes. The ordering is strict:
//! **Clean → Embed → Match → Canonicalize**.
//!
//! ## Design Principles
//!
//! - **Deterministic first**: L0→L1 is purely deterministic. No ML, no API calls.
//! - **Preserve signal**: Middle initials, suffixes, nicknames — never discarded.
//! - **LLMs for confirmation, not generation**: Embeddings retrieve candidates,
//!   LLMs confirm matches. Deterministic steps handle 70%+ of records.
//! - **Immutable layers**: Each layer's output is append-only. Re-processing
//!   creates new versioned outputs, never overwrites.
//! - **The project does not store election data**: It tells users where to get
//!   data, documents every source schema, and provides tools to process it.

pub mod sources;
pub mod pipeline;
pub mod schema;

/// Library version, kept in sync with `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
