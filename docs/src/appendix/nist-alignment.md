# NIST SP 1500-100 Alignment

This appendix maps the pipeline's schema fields to concepts defined in [NIST SP 1500-100 v2](https://pages.nist.gov/ElectionResultsReporting/), the Election Results Common Data Format Specification. The mapping is informational — the pipeline does not emit NIST-compliant XML, but its internal schema was designed with alignment in mind.

## Field mapping

| Pipeline field | NIST SP 1500-100 concept | NIST element | Notes |
|---|---|---|---|
| `contest` | Contest | `CandidateContest` | Candidate races map to `CandidateContest`. |
| `contest` (ballot measure) | Contest | `BallotMeasureContest` | Ballot measures use a separate NIST element. |
| `contest.name` | Contest name | `CandidateContest.Name` | Raw office string before normalization. |
| `contest.canonical_office` | Office | `Office.Name` | L4 normalized office name. |
| `candidate.canonical_first`, `canonical_last` | Candidate | `Candidate.PersonFullName` | Pipeline stores components; NIST stores full name. |
| `candidate.party` | Party | `Party.Abbreviation` | Three-letter codes (DEM, REP, LIB, etc.). |
| `jurisdiction.ocd_id` | Geographic unit | `GpUnit.ExternalIdentifier` | OCD-ID used as the external identifier type. |
| `jurisdiction.county_fips` | Geographic unit | `GpUnit.ExternalIdentifier` | FIPS code, identifier type `fips`. |
| `jurisdiction.state` | Geographic unit | `GpUnit.Type = "state"` | Two-letter USPS abbreviation. |
| `votes.total` | Vote counts | `VoteCounts.Count` | Total votes for a candidate in a contest. |
| `votes.by_mode.election_day` | Vote counts by type | `VoteCounts.CountItemType = "election-day"` | Present in ~33% of records. |
| `votes.by_mode.absentee` | Vote counts by type | `VoteCounts.CountItemType = "absentee"` | Terminology varies by state. |
| `votes.by_mode.early` | Vote counts by type | `VoteCounts.CountItemType = "early"` | Some sources merge into election day. |
| `votes.by_mode.provisional` | Vote counts by type | `VoteCounts.CountItemType = "provisional"` | Timing of inclusion varies. |
| `election.date` | Election | `Election.StartDate` | Single date; no multi-day modeling. |
| `election.type` | Election type | `Election.Type` | Values: `general`, `primary`, `runoff`, `special`. |
| `turnout.registered_voters` | Turnout metadata | `VoteCounts.CountItemType = "total"` on `BallotCounts` | Present in <5% of records. |
| `turnout.ballots_cast` | Turnout metadata | `BallotCounts.BallotsCast` | Same coverage caveat. |
| `contest.district` | Electoral district | `ElectoralDistrict.Name` | District number or name within an office. |

## Concepts not modeled

The following NIST SP 1500-100 concepts have no direct equivalent in the pipeline schema:

- **`RetentionContest`** — Judicial retention elections are classified as `BallotMeasure` with yes/no choices rather than as a distinct contest type.
- **`OrderedContest`** — Ballot ordering is not captured. The pipeline does not model ballot layout.
- **`BallotStyle`** — No ballot style or precinct-to-ballot mapping is maintained.
- **Ranked-choice voting rounds** — `CountItemType` values for RCV rounds (`round-1`, `round-2`, etc.) are not supported. See [Known Limitations](../trust/limitations.md).
- **Overvotes and undervotes** — Tracked as `TurnoutMetadata` contest records at L1, not as NIST `OtherCounts`.

## Pipeline concepts not in NIST

The following pipeline concepts have no NIST equivalent:

- **`provenance.hash`** — SHA-256 hash chain for record integrity. NIST defines no provenance model.
- **`entity_resolution.method`** — Match method metadata (exact, Jaro-Winkler, embedding, LLM). Entity resolution is outside the scope of NIST SP 1500-100.
- **`source.confidence`** — High/medium/low confidence levels. NIST does not model source reliability.
- **Layer identifiers (L0–L4)** — The multi-layer pipeline architecture is specific to this project.