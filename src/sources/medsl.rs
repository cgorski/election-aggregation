//! # MEDSL — MIT Election Data + Science Lab
//!
//! The [MIT Election Data + Science Lab (MEDSL)](https://electionlab.mit.edu/)
//! publishes cleaned, standardized election returns for US federal, state, and
//! some local offices. The primary dataset is the **County Presidential
//! Election Returns** series, but they also publish precinct-level data for
//! recent general elections.
//!
//! ## Download URLs
//!
//! Datasets are hosted on the [Harvard Dataverse](https://dataverse.harvard.edu/)
//! and mirrored on GitHub:
//!
//! - **2018 (precinct-level)**:
//!   <https://dataverse.harvard.edu/dataset.xhtml?persistentId=doi:10.7910/DVN/UBKYRU>
//! - **2020 (precinct-level)**:
//!   <https://dataverse.harvard.edu/dataset.xhtml?persistentId=doi:10.7910/DVN/K7760H>
//! - **2022 (precinct-level)**:
//!   <https://dataverse.harvard.edu/dataset.xhtml?persistentId=doi:10.7910/DVN/2FR5TS>
//! - **GitHub mirror**: <https://github.com/MEDSL/2018-elections-official>,
//!   <https://github.com/MEDSL/2020-elections-official>,
//!   <https://github.com/MEDSL/2022-elections-official>
//!
//! ## CSV Schema (25 columns)
//!
//! The precinct-level CSV files share a common 25-column schema:
//!
//! | # | Column             | Type    | Description                                  |
//! |---|--------------------|---------|----------------------------------------------|
//! |  1| `year`             | int     | Election year (e.g. 2020)                    |
//! |  2| `state`            | string  | State name, title case                       |
//! |  3| `state_po`         | string  | Two-letter USPS state abbreviation           |
//! |  4| `state_fips`       | string  | Two-digit state FIPS code                    |
//! |  5| `state_cen`        | string  | Census state code                            |
//! |  6| `state_ic`         | string  | ICPSR state code                             |
//! |  7| `office`           | string  | Office sought (e.g. "US PRESIDENT")          |
//! |  8| `county_name`      | string  | County name                                  |
//! |  9| `county_fips`      | string  | Five-digit county FIPS code                  |
//! | 10| `jurisdiction_name`| string  | Jurisdiction name (usually county)           |
//! | 11| `jurisdiction_fips`| string  | Jurisdiction FIPS code                       |
//! | 12| `candidate`        | string  | Candidate name as it appears on ballot       |
//! | 13| `district`         | string  | District number or name (if applicable)      |
//! | 14| `dataverse`        | string  | Dataverse category (e.g. "PRESIDENT")        |
//! | 15| `stage`            | string  | Election stage ("GEN", "PRI", "RUN")         |
//! | 16| `special`          | bool    | Whether this is a special election            |
//! | 17| `writein`          | bool    | Whether this is a write-in candidate          |
//! | 18| `mode`             | string  | Vote mode ("TOTAL", "ABSENTEE", etc.)        |
//! | 19| `totalvotes`       | int     | Total votes for this contest in jurisdiction  |
//! | 20| `candidatevotes`   | int     | Votes received by this candidate              |
//! | 21| `version`          | string  | Dataset version timestamp                     |
//! | 22| `readme_check`     | string  | Internal QA flag                              |
//! | 23| `magnitude`        | int     | Number of seats (multi-member districts)      |
//! | 24| `party_detailed`   | string  | Full party name                               |
//! | 25| `party_simplified` | string  | Simplified party ("DEMOCRAT", "REPUBLICAN", …)|
//!
//! ## Known Quirks
//!
//! - Some rows have empty `county_fips` for statewide or at-large contests.
//! - `candidate` values are not normalized across years — same person may
//!   appear as "JOSEPH R BIDEN JR" in one file and "BIDEN, JOSEPH R." in another.
//! - The `mode` column granularity varies by state; many states only report "TOTAL".

/// Placeholder for the MEDSL data source connector.
///
/// Will eventually handle downloading and parsing precinct-level
/// election returns from the MIT Election Data + Science Lab.
pub struct MedslSource;
