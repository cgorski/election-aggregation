//! Five-layer election data pipeline.
//!
//! The pipeline processes election data through five immutable, append-only layers.
//! Each layer depends on all prior layers, and every record carries a hash chain
//! back to the original source bytes.
//!
//! # Layers
//!
//! | Layer | Name        | Description                                                        |
//! |-------|-------------|--------------------------------------------------------------------|
//! | L0    | **Raw**       | Byte-identical copies of source files with acquisition manifests. |
//! | L1    | **Cleaned**   | Parsed, structured records with all name components preserved.   |
//! | L2    | **Embedded**  | Vector embeddings attached for fuzzy matching and classification.|
//! | L3    | **Matched**   | Entity-resolved records with candidate and contest identifiers.  |
//! | L4    | **Canonical** | Authoritative names, temporal chains, and verification status.   |
//!
//! # Processing Order
//!
//! The ordering is strict: **L0 → L1 → L2 → L3 → L4**.
//!
//! - **L0 → L1** is purely deterministic (no ML, no API calls).
//! - **L1 → L2** computes embeddings (may use an embedding model).
//! - **L2 → L3** performs entity resolution via retrieval + confirmation.
//! - **L3 → L4** assigns canonical names and builds temporal chains.
//!
//! Re-processing a layer creates a new versioned output; prior outputs are
//! never overwritten.

pub mod l0;
pub mod l1;
pub mod l2;
pub mod l3;
pub mod l4;
