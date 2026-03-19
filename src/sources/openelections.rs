//! # OpenElections Project
//!
//! The [OpenElections](http://www.openelections.net/) project is a volunteer-driven
//! effort to collect, clean, and publish certified election results for all US states.
//! Data is organized into per-state GitHub repositories under the
//! [`openelections`](https://github.com/openelections) organization.
//!
//! ## Repository structure
//!
//! Each state has its own repo following the naming convention:
//!
//! - `openelections-data-nc` — North Carolina
//! - `openelections-data-pa` — Pennsylvania
//! - `openelections-data-ga` — Georgia
//! - etc.
//!
//! Within each repo, CSV files are organized by year and election type, e.g.:
//!
//! ```text
//! 20201103__nc__general__precinct.csv
//! 20220510__pa__primary__county.csv
//! ```
//!
//! ## CSV schema (7+ columns, varies by state)
//!
//! The core columns present in most files are:
//!
//! | # | Column        | Type   | Description                                    |
//! |---|---------------|--------|------------------------------------------------|
//! | 1 | `county`      | String | County name                                    |
//! | 2 | `precinct`    | String | Precinct name or code                          |
//! | 3 | `office`      | String | Office contested (e.g. "US Senate")            |
//! | 4 | `district`    | String | District number or name (may be blank)         |
//! | 5 | `party`       | String | Party abbreviation (e.g. "DEM", "REP")         |
//! | 6 | `candidate`   | String | Candidate name (format varies by state)        |
//! | 7 | `votes`       | Int    | Vote count                                     |
//!
//! Additional columns that may appear depending on the state:
//!
//! - `election_day`, `absentee`, `provisional`, `early_voting` — vote mode breakdowns
//! - `winner` — boolean or "Y"/"N" flag
//! - `total_votes` — aggregate across modes
//!
//! ## Coverage
//!
//! Coverage varies significantly by state. Some states have data going back to
//! 2000, while others only have a few recent cycles. Approximately 8–10 states
//! have precinct-level data suitable for aggregation. Check each state repo's
//! README for specifics.
//!
//! ## Data quality notes
//!
//! - **No standard schema**: Column names and ordering differ across states and
//!   even across files within the same state repo.
//! - **Candidate name format varies**: Some states use "Last, First" while
//!   others use "First Last". Suffixes and middle names are inconsistent.
//! - **Encoding**: Files are generally UTF-8 but some older files may have
//!   Latin-1 or Windows-1252 characters.
//! - **Duplicates**: Some repos contain both raw and cleaned versions of the
//!   same election; care must be taken to avoid double-counting.

/// Stub source connector for the OpenElections project.
///
/// Will eventually handle discovery and parsing of per-state CSV files
/// from the OpenElections GitHub repositories.
pub struct OpenElectionsSource;
