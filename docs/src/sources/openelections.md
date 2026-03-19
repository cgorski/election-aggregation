# OpenElections

The [OpenElections](http://www.openelections.net/) project is a volunteer-driven effort to collect, clean, and publish certified US election results as CSV files. Data is organized into per-state GitHub repositories under the [`openelections`](https://github.com/openelections) organization.

## What OpenElections provides

Precinct-level and county-level election results, parsed from official state and county sources into CSV format. Coverage varies by state — some repositories have data back to 2000, others have only one or two recent cycles. Approximately 8 states have precinct-level 2022 general election data suitable for aggregation.

States with usable precinct-level data for recent cycles include FL, GA, MI, OH, PA, and TX. Each state repository is independent, maintained by different volunteers, with different levels of completeness.

## Repository structure

Each state has its own repo:

- `openelections-data-fl` — Florida
- `openelections-data-ga` — Georgia
- `openelections-data-pa` — Pennsylvania
- etc.

Files follow a naming convention that encodes the election date, state, type, and granularity:

```
{YYYYMMDD}__{state}__{type}__{granularity}.csv
```

Examples:

| Filename | Meaning |
|----------|---------|
| `20221108__fl__general__precinct.csv` | 2022 FL general, precinct-level |
| `20220510__pa__primary__county.csv` | 2022 PA primary, county-level |
| `20201103__ga__general__precinct.csv` | 2020 GA general, precinct-level |

Some repos include both raw and cleaned versions. Files with `raw` in the name are unprocessed source dumps. Prefer files without the `raw` prefix.

## Core schema (7+ columns)

The project does not enforce a single schema. Most files share a 7-column core:

| Column | Type | Description |
|--------|------|-------------|
| `county` | string | County name |
| `precinct` | string | Precinct name or code |
| `office` | string | Office contested |
| `district` | string | District number or name (may be blank) |
| `party` | string | Party abbreviation |
| `candidate` | string | Candidate name |
| `votes` | integer | Vote count |

Additional columns appear in some states:

- `election_day`, `absentee`, `provisional`, `early_voting` — vote mode breakdowns
- `winner` — boolean or `Y`/`N` flag
- `total_votes` — aggregate across modes

Column names and ordering differ across states and sometimes across files within the same state repo.

## Schema variation by state

| State | Extra columns | Name format | Notes |
|-------|--------------|-------------|-------|
| FL | `election_day`, `absentee`, `early_voting` | `Last, First` | Includes "Registered Voters" metadata rows (6,013 in 2022) |
| GA | `total_votes` | `First Last` | Precinct names vary by county |
| PA | none beyond core | `First Last` | Some files county-level only |
| OH | `early_voting`, `absentee` | `Last, First` | Inconsistent across counties |

## Non-candidate rows

Florida files include metadata rows that are not contest results:

| `office` value | Meaning |
|----------------|---------|
| `Registered Voters` | Voter registration count — 67.9% of "other" rows in initial FL processing |
| `Ballots Cast` | Turnout count |

These must be extracted as turnout metadata during L1 parsing, not treated as contests.

## Access method

Data is accessed by cloning the per-state Git repository:

```sh
git clone https://github.com/openelections/openelections-data-fl.git
git clone https://github.com/openelections/openelections-data-ga.git
git clone https://github.com/openelections/openelections-data-pa.git
```

There is no bulk download endpoint. Each state repo must be cloned individually.

## Data quality

Quality varies by state and volunteer. Known issues:

- **No standard schema.** Column names differ across states and files. Parsers must handle each state separately.
- **Candidate name format varies.** Some states use `Last, First`. Others use `First Last`. Suffixes and middle names are inconsistent.
- **Encoding.** Most files are UTF-8. Some older files contain Latin-1 or Windows-1252 characters.
- **Duplicates.** Some repos contain both raw and cleaned versions of the same election. Ingest only one to avoid double-counting.
- **Incomplete coverage.** A state repo existing does not mean it has precinct-level data for all cycles.

## Cross-source overlap

OpenElections FL overlaps with MEDSL FL for the 2022 general election. This overlap is useful for validation but has not been systematically compared at the same depth as the MEDSL–NC SBE comparison (640 contests, 90.5% vote match). The FL overlap is a planned validation target.

## Value in the pipeline

OpenElections fills gaps where MEDSL coverage is thin or where vote mode breakdowns are available. Florida's vote mode columns (election day, absentee, early voting) provide signal that MEDSL's Florida file lacks. The community-curated nature means data may appear for states or cycles before MEDSL publishes its cleaned version.

The tradeoff is consistency: every state requires its own parser branch at L1.