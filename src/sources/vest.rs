//! # VEST — Voting and Election Science Team
//!
//! The [Voting and Election Science Team (VEST)](https://election.lab.ufl.edu/precinct-data/)
//! publishes precinct-level election shapefiles for all 50 states. Each state
//! has one shapefile per redistricting cycle, with results for multiple contests
//! encoded as attribute columns.
//!
//! ## Data Location
//!
//! VEST shapefiles are archived on the Harvard Dataverse:
//!
//! - **2020 data**: <https://dataverse.harvard.edu/dataset.xhtml?persistentId=doi:10.7910/DVN/K7760H>
//! - **2018 data**: <https://dataverse.harvard.edu/dataset.xhtml?persistentId=doi:10.7910/DVN/UBKYRU>
//! - **2016 data**: <https://dataverse.harvard.edu/dataset.xhtml?persistentId=doi:10.7910/DVN/NH5S2I>
//!
//! Each dataset is typically distributed as a ZIP archive containing `.shp`,
//! `.dbf`, `.shx`, `.prj`, and `.cpg` files.
//!
//! ## Column Encoding Convention
//!
//! VEST uses a compact column naming convention that encodes the election year,
//! office, party, and candidate surname into a single identifier. The format is:
//!
//! ```text
//! G{YY}{OFF}{PTY}{NAME}
//! ```
//!
//! Where:
//!
//! - `G` — General election (or `P` for primary, `R` for runoff)
//! - `YY` — Two-digit election year (e.g., `20` for 2020)
//! - `OFF` — Office code: `PRE` (President), `USS` (US Senate), `USH` (US House),
//!   `GOV` (Governor), `SOS` (Secretary of State), `AG` (Attorney General), etc.
//! - `PTY` — Party code: `R` (Republican), `D` (Democrat), `L` (Libertarian),
//!   `G` (Green), `O` (Other)
//! - `NAME` — Abbreviated candidate surname (e.g., `TRU` for Trump, `BID` for Biden)
//!
//! ### Examples
//!
//! | Column         | Meaning                                       |
//! |----------------|-----------------------------------------------|
//! | `G20PRERTRU`   | 2020 General, President, Republican, Trump     |
//! | `G20PREDBID`   | 2020 General, President, Democrat, Biden       |
//! | `G20USSRPER`   | 2020 General, US Senate, Republican, Perdue    |
//! | `G18GOVDABO`   | 2018 General, Governor, Democrat, Abrams       |
//!
//! ## Schema Notes
//!
//! The `.dbf` attribute table typically contains:
//!
//! - `STATEFP20` / `COUNTYFP20` / `VTDST20` — FIPS codes for state, county, precinct
//! - `NAME20` — Human-readable precinct name
//! - `ALAND20` / `AWATER20` — Land and water area in square meters
//! - One column per contest-party-candidate combination (see encoding above)
//! - Values are raw vote counts (integers)
//!
//! ## Known Quirks
//!
//! - Column names are truncated to 10 characters (dBASE III limitation), which
//!   sometimes makes candidate surnames ambiguous.
//! - Some states use custom office codes beyond the standard set.
//! - Precinct boundaries may not align across election years due to redistricting.
//! - Shapefiles can be very large (hundreds of MB per state).

/// Placeholder for the VEST shapefile data source.
///
/// Will handle downloading, parsing, and decoding VEST precinct-level
/// shapefiles and their encoded column names into structured election records.
pub struct VestSource;
