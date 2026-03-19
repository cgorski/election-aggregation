//! # FEC — Federal Election Commission Candidate Master Files
//!
//! The FEC publishes bulk data files containing information about every
//! registered federal candidate (President, Senate, House). These files
//! are the authoritative source for candidate IDs, party affiliation,
//! office sought, district, and incumbent/challenger status.
//!
//! ## Download
//!
//! Bulk data is available at:
//! - <https://www.fec.gov/data/browse-data/?tab=bulk-data>
//! - Direct link pattern: `https://www.fec.gov/files/bulk-downloads/{YEAR}/cn.zip`
//!   where `{YEAR}` is a two- or four-digit election cycle year (e.g., `2024`).
//!
//! The candidate master file is named `cn.txt` inside the ZIP archive.
//!
//! ## Schema
//!
//! The candidate master file (`cn.txt`) is pipe-delimited (`|`) with **15 columns**
//! and no header row:
//!
//! | # | Field                    | Description                              |
//! |---|--------------------------|------------------------------------------|
//! | 1 | `CAND_ID`               | Candidate ID (e.g., `H0NC09072`)        |
//! | 2 | `CAND_NAME`             | Candidate name (LAST, FIRST MIDDLE)      |
//! | 3 | `CAND_PTY_AFFILIATION`  | Party code (e.g., `REP`, `DEM`)          |
//! | 4 | `CAND_ELECTION_YR`      | Election year                            |
//! | 5 | `CAND_OFFICE_ST`        | State of candidacy (2-letter FIPS)       |
//! | 6 | `CAND_OFFICE`           | Office: `H` (House), `S` (Senate), `P`  |
//! | 7 | `CAND_OFFICE_DISTRICT`  | Congressional district (`00` for Senate) |
//! | 8 | `CAND_ICI`              | Incumbent/Challenger/Open: `I`/`C`/`O`  |
//! | 9 | `CAND_STATUS`           | Status code (`C`, `F`, `N`, `P`)         |
//! |10 | `CAND_PCC`              | Principal campaign committee ID          |
//! |11 | `CAND_ST1`              | Mailing address street                   |
//! |12 | `CAND_ST2`              | Mailing address street 2                 |
//! |13 | `CAND_CITY`             | Mailing address city                     |
//! |14 | `CAND_ST`               | Mailing address state                    |
//! |15 | `CAND_ZIP`              | Mailing address ZIP code                 |
//!
//! ## Coverage
//!
//! - **Scope**: All registered federal candidates (President, Senate, House)
//! - **Years**: 1980–present (bulk files available per election cycle)
//! - **Update frequency**: Nightly
//!
//! ## Usage in the Pipeline
//!
//! FEC candidate master records serve as a **reference source** in L3 (Matching).
//! The `CAND_ID` provides a stable identifier for federal candidates that can be
//! linked to vote totals from MEDSL, OpenElections, or state sources. The
//! `CAND_NAME` field is parsed during L1 to extract last name, first name,
//! middle name, suffix, and nickname components for entity resolution.

/// Placeholder source handle for FEC candidate master file data.
pub struct FecSource;
