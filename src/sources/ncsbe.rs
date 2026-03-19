//! # North Carolina State Board of Elections (NC SBE)
//!
//! The NC State Board of Elections publishes precinct-level election results
//! for all contests in North Carolina, spanning general and primary elections
//! from 2006 through 2024 (approximately 10 election cycles).
//!
//! ## Format
//!
//! Results are distributed as **tab-delimited** text files (`.txt`), one file
//! per election. Each file contains a 15-column schema:
//!
//! | # | Column              | Description                                  |
//! |---|---------------------|----------------------------------------------|
//! | 1 | `County`            | County name (e.g. "WAKE")                    |
//! | 2 | `Election Date`     | Date of the election (MM/DD/YYYY)            |
//! | 3 | `Precinct Code`     | Numeric precinct identifier                  |
//! | 4 | `Precinct Name`     | Human-readable precinct name                 |
//! | 5 | `Contest Group ID`  | Numeric contest group identifier             |
//! | 6 | `Contest Type`      | Type code (e.g. "G" for general)             |
//! | 7 | `Contest Name`      | Full contest name as it appeared on ballot   |
//! | 8 | `Choice`            | Candidate or ballot measure choice name      |
//! | 9 | `Choice Party`      | Party abbreviation (e.g. "REP", "DEM")       |
//! |10 | `Vote For`          | Number of choices allowed                    |
//! |11 | `Election Day`      | Election day vote count                      |
//! |12 | `One Stop`          | Early voting (one-stop) vote count           |
//! |13 | `Absentee by Mail`  | Mail-in absentee vote count                  |
//! |14 | `Provisional`       | Provisional ballot vote count                |
//! |15 | `Total Votes`       | Sum of all vote modes                        |
//!
//! ## Download URL Pattern
//!
//! Results are available from the NC SBE results portal:
//!
//! ```text
//! https://er.ncsbe.gov/downloads.html
//! ```
//!
//! Direct file downloads follow the pattern:
//!
//! ```text
//! https://er.ncsbe.gov/resultsfiles/results_pct_<YYYYMMDD>.zip
//! ```
//!
//! For example, the 2022 general election results:
//!
//! ```text
//! https://er.ncsbe.gov/resultsfiles/results_pct_20221108.zip
//! ```
//!
//! ## Date Range
//!
//! - **Earliest available**: 2006 general election
//! - **Latest available**: 2024 general election
//! - Covers both primary and general elections for each cycle
//!
//! ## Known Quirks
//!
//! - Files use **tab** as the delimiter, not comma.
//! - County names are **ALL CAPS** (e.g. "WAKE", "MECKLENBURG").
//! - The `Choice` field may contain suffixes, nicknames, and middle initials
//!   that vary across election years for the same candidate.
//! - Provisional vote counts may be zero in early result files and updated later.

/// Placeholder source handle for NC State Board of Elections data.
pub struct NcsbeSource;
