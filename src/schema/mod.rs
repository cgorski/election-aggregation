//! Unified election record schema definitions.
//!
//! This module defines the canonical schema that all election data sources
//! are ultimately normalized into. The schema is designed to capture the
//! full breadth of US election results — from presidential races down to
//! local school board contests — in a single, consistent representation.
//!
//! # Core Concepts
//!
//! - **Contest**: A single race or ballot measure (e.g., "US Senate, NC, 2022").
//! - **Candidate**: A person or option (e.g., "Write-In", "Yes/No" for measures).
//! - **Result**: A vote count for a specific candidate in a specific contest
//!   within a specific geographic unit.
//! - **Geography**: A hierarchy from state → county → precinct, identified by
//!   FIPS codes where available.
//!
//! # Design Principles
//!
//! - Every field that *any* source provides is representable, even if most
//!   sources leave it null.
//! - Name components (first, middle, last, suffix, nickname) are stored
//!   separately — never flattened into a single string at the schema level.
//! - Party is stored as both a raw string (from the source) and a normalized
//!   enum, because sources are wildly inconsistent ("DEM", "Democrat",
//!   "Democratic", "D").
//! - Vote counts distinguish between election-day, early, absentee/mail,
//!   and provisional where the source provides that breakdown.
