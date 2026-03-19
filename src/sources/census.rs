//! # US Census Bureau — FIPS Reference Files
//!
//! The Census Bureau publishes authoritative FIPS (Federal Information
//! Processing Standards) code files that map numeric codes to geographic
//! entity names at every level of the hierarchy:
//!
//! - **State FIPS codes** (2-digit): e.g. `37` → North Carolina
//! - **County FIPS codes** (5-digit: state + county): e.g. `37183` → Wake County, NC
//! - **County subdivision codes** (10-digit): townships, boroughs, etc.
//! - **Place codes** (7-digit): incorporated cities, CDPs
//!
//! ## Download URLs
//!
//! National FIPS code files are available from:
//!
//! - State codes: <https://www2.census.gov/geo/docs/reference/state.txt>
//! - County codes: <https://www2.census.gov/geo/docs/reference/codes2020/national_county2020.txt>
//! - County subdivisions: <https://www2.census.gov/geo/docs/reference/codes2020/national_cousub2020.txt>
//! - Places: <https://www2.census.gov/geo/docs/reference/codes2020/national_place2020.txt>
//!
//! The Geocodes file provides a single hierarchical lookup:
//! <https://www2.census.gov/programs-surveys/popest/geographies/2020/all-geocodes-v2020.xlsx>
//!
//! ## Schema (county file example)
//!
//! The national county file is pipe-delimited (`|`) with the following columns:
//!
//! | Column        | Description                          |
//! |---------------|--------------------------------------|
//! | `STATE`       | 2-digit state FIPS code              |
//! | `STATEFP`     | Same as STATE (varies by file)       |
//! | `COUNTYFP`    | 3-digit county FIPS code             |
//! | `COUNTYNS`    | ANSI feature code                    |
//! | `COUNTYNAME`  | Full county name                     |
//! | `CLASSFP`     | FIPS class code (H1, H4, H6, etc.)  |
//! | `FUNCSTAT`    | Functional status (A=Active, etc.)   |
//!
//! ## Usage in the Pipeline
//!
//! FIPS codes are the primary geographic join key across all election data
//! sources. Census reference files provide the authoritative mapping from
//! codes to names, enabling normalization of inconsistent geographic labels
//! across MEDSL, NC SBE, VEST shapefiles, and other sources.

/// Handle for the US Census Bureau FIPS reference file source.
pub struct CensusSource;
