//! Data source definitions and acquisition logic.
//!
//! Each submodule corresponds to one election data source. Every source
//! module documents:
//!
//! - Where to download the data
//! - The source schema (columns, types, quirks)
//! - How to parse the source format into L1 records
//! - Known data quality issues
//!
//! # Supported Sources
//!
//! | Module          | Source                              | Coverage                        |
//! |-----------------|-------------------------------------|---------------------------------|
//! | [`medsl`]       | MIT Election Data + Science Lab     | 50 states + DC, 2018/2020/2022  |
//! | [`ncsbe`]       | NC State Board of Elections          | NC, 2006–2024 (10 cycles)       |
//! | [`openelections`] | OpenElections Project             | ~8 states, varies by year       |
//! | [`clarity`]     | Clarity/Scytl ENR sites             | ~1,000+ jurisdictions           |
//! | [`vest`]        | Voting & Election Science Team      | 50 states, shapefiles           |
//! | [`census`]      | US Census Bureau reference files    | National FIPS codes             |
//! | [`fec`]         | Federal Election Commission         | Federal candidates              |

pub mod medsl;
pub mod ncsbe;
pub mod openelections;
pub mod clarity;
pub mod vest;
pub mod census;
pub mod fec;
