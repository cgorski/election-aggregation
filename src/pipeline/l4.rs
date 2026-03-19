//! # L4 — Canonical Layer
//!
//! The final layer of the pipeline. L4 produces the authoritative,
//! publication-ready dataset by assigning canonical names, building
//! temporal chains across election cycles, and attaching verification
//! metadata.
//!
//! ## Inputs
//!
//! - **L3 matched records** (`L3Record`): Entity-resolved records with
//!   stable candidate and contest identifiers.
//! - **Reference data**: Census FIPS files, FEC candidate master files,
//!   and any manually curated override tables.
//!
//! ## Outputs
//!
//! - **L4 canonical records** (`L4Record`): Each record carries:
//!   - A canonical candidate name (preferred display form).
//!   - A canonical contest/office name.
//!   - A temporal chain linking the same candidate across election
//!     cycles (e.g., a council member who ran in 2018 and 2022).
//!   - A verification status indicating whether the record was
//!     confirmed by a second source, an LLM review, or is unverified.
//!   - The full hash chain back to the L0 source bytes.
//!
//! ## Processing Steps
//!
//! 1. **Name canonicalization**: Select the preferred display form for
//!    each candidate and contest from the set of observed variants.
//! 2. **Temporal chaining**: Link entity IDs across election years to
//!    build longitudinal candidate histories.
//! 3. **Verification tagging**: Mark each record with its verification
//!    level (multi-source confirmed, LLM-confirmed, single-source
//!    unverified).
//! 4. **Schema export**: Emit the final unified schema suitable for
//!    downstream analysis, visualization, or API serving.

/// A single canonical election record — the final output of the pipeline.
pub struct L4Record;
