//! # L2 — Embedded Layer
//!
//! The Embedded layer enriches cleaned L1 records with vector embeddings
//! that enable fuzzy matching and semantic similarity search across
//! heterogeneous election data sources.
//!
//! ## Purpose
//!
//! Raw text fields — candidate names, contest titles, jurisdiction names —
//! vary wildly across sources. Deterministic normalization (L1) handles
//! obvious differences, but many ambiguities remain:
//!
//! - "JOHN A. SMITH" vs "Smith, John Andrew"
//! - "County Commissioner District 3" vs "Board of County Commissioners, Dist. III"
//! - "Town of Holly Springs" vs "Holly Springs Municipality"
//!
//! L2 computes dense vector embeddings for these text fields so that
//! downstream matching (L3) can retrieve candidate pairs efficiently
//! using approximate nearest-neighbor search.
//!
//! ## Inputs
//!
//! - **L1 records** ([`super::l1::L1Record`]): Cleaned, structured records
//!   with normalized name components and contest fields.
//!
//! ## Outputs
//!
//! - **L2 records** ([`L2Record`]): Each L1 record augmented with:
//!   - `candidate_name_embedding`: Vector for the full candidate name
//!   - `contest_name_embedding`: Vector for the contest/office title
//!   - `jurisdiction_embedding`: Vector for the jurisdiction name
//!   - `embedding_model`: Identifier of the model used (for reproducibility)
//!   - `embedding_version`: Version tag so re-embedding creates a new generation
//!
//! ## Design Notes
//!
//! - Embedding is the **first non-deterministic step** in the pipeline.
//!   Different models or model versions will produce different vectors.
//! - Each embedding record stores the model identifier and version so that
//!   downstream consumers know which vectors are comparable.
//! - Embeddings are computed in batches for throughput. The batch size is
//!   configurable but defaults to 256 records.
//! - L2 outputs are **append-only**: re-embedding with a new model creates
//!   a new versioned output alongside the old one.

/// A single L2 record: an L1 record augmented with vector embeddings
/// for candidate name, contest name, and jurisdiction fields.
pub struct L2Record;
