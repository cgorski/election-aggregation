//! # L3 — Matched Records
//!
//! The matching layer performs **entity resolution** across records that were
//! embedded in L2. Two records "match" when they refer to the same real-world
//! candidate running in the same real-world contest, even if the source data
//! spells names differently, uses different office titles, or assigns
//! different district identifiers.
//!
//! ## Inputs
//!
//! - [`super::l2::L2Record`] — embedded records carrying vector representations
//!   of candidate names, office titles, and geographic identifiers.
//!
//! ## Outputs
//!
//! - [`L3Record`] — each record is annotated with:
//!   - A **candidate cluster ID** linking all source mentions of the same person.
//!   - A **contest cluster ID** linking all source mentions of the same race.
//!   - A **match confidence score** (0.0–1.0) indicating how certain the link is.
//!   - The method used (`Deterministic`, `Embedding`, `LlmConfirmed`).
//!
//! ## Strategy
//!
//! Matching proceeds in three tiers, each more expensive than the last:
//!
//! 1. **Deterministic blocking** — exact match on (state FIPS, year, normalized
//!    last name, office keyword). This resolves 70%+ of records with zero
//!    ambiguity and no model calls.
//!
//! 2. **Embedding nearest-neighbor** — for records that survive tier 1, use
//!    cosine similarity on L2 vectors to propose candidate pairs above a
//!    threshold (e.g. 0.92). Cluster with single-linkage or DBSCAN.
//!
//! 3. **LLM confirmation** — ambiguous pairs (similarity 0.85–0.92) are sent
//!    to an LLM with structured prompts: "Are these the same person running
//!    for the same office?" The LLM does not generate names — it only confirms
//!    or rejects proposed matches.
//!
//! ## Design Notes
//!
//! - Every match decision is logged with its method and score so that
//!   downstream consumers can filter by confidence.
//! - Cluster IDs are content-addressed hashes of the member record IDs,
//!   making them stable across re-runs with identical inputs.
//! - No record is ever discarded. Unmatched records form singleton clusters
//!   and flow through to L4 for manual review.

/// A matched record carrying entity-resolution cluster assignments.
///
/// Each `L3Record` links back to its L2 source and adds cluster IDs
/// for the resolved candidate and contest entities.
pub struct L3Record;
