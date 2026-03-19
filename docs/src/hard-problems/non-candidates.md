# Non-Candidate Records

Not every row in an election results file is a candidate. Sources routinely embed turnout metadata, ballot measure choices, vote quality indicators, and aggregation artifacts alongside candidate results — using the same columns, the same format, and no reliable flag to distinguish them.

If your system treats every row as a candidate, you will create entity records for people named "Registered Voters", "For", "BLANK", and "TOTAL VOTES". The L4 LLM audit in our prototype caught exactly this: "For" and "Against" were classified as person entities. They are not people.

## The Four Categories

### 1. Turnout Metadata

Rows recording registration and participation counts at the precinct level:

| Pseudo-candidate | Meaning | Source |
|------------------|---------|--------|
| Registered Voters | Total registered voters in precinct | FL OpenElections, NC SBE |
| Ballots Cast | Total ballots submitted | FL OpenElections, NC SBE |
| Cards Cast | Total ballot cards (may differ from ballots in multi-card elections) | FL OpenElections |

Florida OpenElections is the most prolific source. Of the "other" records in our FL 2022 ingest, **6,013 rows are "Registered Voters"** — accounting for 67.9% of all non-candidate records in that source. These are not errors in the source data. They are genuine turnout figures published alongside contest results in the same file format.

### 2. Ballot Measure Choices

Rows representing choices on referenda, bond issues, and constitutional amendments:

| Pseudo-candidate | Meaning | Source |
|------------------|---------|--------|
| For | Yes vote on ballot measure | OpenElections, MEDSL |
| Against | No vote on ballot measure | OpenElections, MEDSL |
| Yes | Yes vote on ballot measure | NC SBE, MEDSL |
| No | No vote on ballot measure | NC SBE, MEDSL |

These are legitimate vote counts — but the "candidate" is not a person. Detection requires examining both the candidate name (a single common word) and the contest name (bond, referendum, amendment, proposition). See [Ballot Measure Choices](./non-candidates-ballot-measures.md).

### 3. Vote Quality Indicators

Rows recording ballots that did not produce a valid vote for any candidate:

| Pseudo-candidate | Meaning | Source |
|------------------|---------|--------|
| Over Votes | Voter selected more candidates than allowed | MEDSL, NC SBE |
| Under Votes | Voter selected fewer candidates than allowed | MEDSL, NC SBE |
| BLANK | No selection made (Maine's term for undervote) | MEDSL (ME) |
| Write-in | Aggregate write-in count (no specific candidate) | Multiple sources |

Over votes and under votes are important data quality signals. A contest with 15% over votes may indicate a confusing ballot design. But they are not candidates and must not be counted as such.

### 4. Aggregation Artifacts

Rows that are computational summaries, not individual results:

| Pseudo-candidate | Meaning | Source |
|------------------|---------|--------|
| TOTAL VOTES | Sum of all candidates in the contest | MEDSL (UT) |
| Scattering | Aggregate of write-in candidates below reporting threshold | MEDSL (IA, MN) |
| TOTAL | Another sum variant | OpenElections |

These rows are redundant with the candidate-level data. Including them double-counts votes and inflates totals.

## The Detection Strategy

Non-candidate records are detected at L1 — the earliest possible point. The principle is **extract before filter**: non-candidate rows often contain valuable information (registered voter counts, undervote rates) that should be captured in the correct schema object before the row is excluded from contest analysis.

Detection uses a three-part check:

1. **Exact match on candidate name.** A lookup table of ~40 known pseudo-candidate strings: "Registered Voters", "Ballots Cast", "Over Votes", "Under Votes", "BLANK", "TOTAL VOTES", "Scattering", "For", "Against", "Yes", "No", etc.

2. **Contest name pattern.** For ambiguous names like "For" and "Against", check whether the contest name contains ballot measure keywords: bond, referendum, amendment, proposition, measure, question, initiative, charter.

3. **Source-specific rules.** Some sources use unique pseudo-candidates. Maine uses "BLANK". Iowa uses "Scattering". Utah includes "TOTAL VOTES" rows. Each source parser knows its own ghosts.

## Routing

Detected non-candidate records are routed to the appropriate schema object:

| Category | Route to | Schema type |
|----------|----------|-------------|
| Turnout metadata | `TurnoutMetadata` | Attached to sibling precinct records |
| Ballot measure choices | `BallotMeasure` | `MeasureChoice` with For/Against/Yes/No |
| Vote quality indicators | `VoteQuality` | Attached to parent contest record |
| Aggregation artifacts | Discarded | Redundant with candidate-level sums |

Records routed to `TurnoutMetadata` and `VoteQuality` are preserved in the L1 output — they are valuable data, just not candidate data. Aggregation artifacts are discarded with a note in the cleaning report.

## What Happens Without Detection

If non-candidate rows pass through to L2 and L3:

- "Registered Voters" gets an embedding vector, a candidate entity ID, and appears in 6,013 precinct-level records as the most prolific "candidate" in Florida.
- "For" and "Against" become person entities. The L4 LLM audit flagged exactly this in our prototype: "'For' is not a plausible person name."
- "TOTAL VOTES" inflates vote counts when aggregated, because the total row is summed alongside the individual candidate rows.
- "Over Votes" appears as a candidate who received votes in every contest — the busiest politician in America.

Detection at L1 prevents all of these downstream errors.

## Sub-Chapters

- [Registered Voters, Ballots Cast, Over/Under Votes](./non-candidates-metadata.md) — turnout and vote quality rows, the "extract before filter" principle
- [Ballot Measure Choices: For/Against/Yes/No](./non-candidates-ballot-measures.md) — detecting ballot measures from candidate name + contest name patterns