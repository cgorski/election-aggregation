//! Clarity / Scytl Election Night Reporting (ENR) source.
//!
//! # Overview
//!
//! Clarity (now part of Scytl / CivicPlus) powers the Election Night Reporting
//! websites for over 1,000 US jurisdictions — counties, cities, and some states.
//! Each jurisdiction runs its own ENR instance serving live and final results in
//! a structured XML (and sometimes JSON) format.
//!
//! # Data Format
//!
//! Results are published as XML feeds. The key elements include:
//!
//! - `<ElectionResult>` — root element with election metadata
//! - `<Contest>` — one per race (e.g., "US Senate", "County Commissioner Dist 3")
//! - `<Choice>` — one per candidate or ballot option within a contest
//! - `<VoteType>` — breakdown by vote method (Election Day, Absentee, Early, Provisional)
//! - `<Precinct>` — precinct-level results when available
//!
//! Typical fields per result row:
//!
//! | Field             | Description                                    |
//! |-------------------|------------------------------------------------|
//! | `contest_name`    | Name of the race                               |
//! | `choice_name`     | Candidate or option name                       |
//! | `party`           | Party affiliation (if provided)                |
//! | `vote_type`       | Election Day / Absentee / Early / Provisional  |
//! | `votes`           | Vote count for this choice + vote type          |
//! | `precinct_name`   | Precinct identifier (when precinct-level)      |
//! | `total_votes`     | Total votes cast in the contest                |
//! | `ballots_cast`    | Total ballots cast (may differ from votes)     |
//! | `registered_voters` | Registered voter count for the jurisdiction  |
//!
//! # URL Discovery
//!
//! Clarity ENR sites follow a predictable URL pattern:
//!
//! ```text
//! https://results.enr.clarityelections.com/<state>/<county>/<election_id>/Web02/en/summary.html
//! ```
//!
//! The underlying data feeds are typically at:
//!
//! ```text
//! https://results.enr.clarityelections.com/<state>/<county>/<election_id>/reports/detailxml.zip
//! https://results.enr.clarityelections.com/<state>/<county>/<election_id>/json/en/summary.json
//! ```
//!
//! To discover active ENR sites:
//!
//! 1. Start from a known Clarity URL for a county (often linked from the county
//!    elections office website).
//! 2. The `<election_id>` is a numeric identifier that increments per election.
//! 3. The `detailxml.zip` file contains `detail.xml` with full precinct-level
//!    results once polls close.
//! 4. Some jurisdictions also expose a JSON summary endpoint for lighter polling.
//!
//! # Coverage
//!
//! - **Jurisdictions**: ~1,000+ counties and municipalities across the US
//! - **Election types**: General, Primary, Runoff, Special
//! - **Granularity**: Typically precinct-level with vote-type breakdowns
//! - **Timeliness**: Live on election night; final (certified) results may
//!   remain on the site for weeks or months before being taken down
//!
//! # Quirks
//!
//! - URLs are ephemeral: old election results may be removed without notice.
//! - XML schema is not formally published and varies slightly across versions.
//! - Candidate names may include party abbreviations inline (e.g., "John Smith (REP)").
//! - Some jurisdictions use Clarity for primaries but not generals, or vice versa.
//! - The `detailxml.zip` may not appear until after polls close.

/// Placeholder for the Clarity/Scytl ENR data source.
pub struct ClaritySource;
