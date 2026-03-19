# Registered Voters, Ballots Cast, Over/Under Votes

Some election data files embed turnout metadata and vote-quality indicators directly alongside candidate results. A row labeled "Registered Voters" is not a contest — it is a count of eligible voters in that precinct. A row labeled "Over Votes" is not a candidate — it is a count of ballots where the voter marked too many choices.

These rows are valuable. They are also poison if treated as candidates.

## The Four Categories

| Label | What it means | Found in |
|-------|--------------|----------|
| Registered Voters | Eligible voters in the precinct | NC SBE, FL OpenElections |
| Ballots Cast | Ballots submitted (any contest) | NC SBE, some MEDSL records |
| Over Votes | Ballots with too many selections for a contest | NC SBE, ME, UT |
| Under Votes | Contests where the voter made no selection | NC SBE, ME, UT |

NC SBE includes all four in every precinct file. MEDSL includes over/under votes for some states but not others. OpenElections varies by state and contributor. There is no standard.

## The Extract-Before-Filter Principle

The instinct is to filter these rows out immediately — they are not candidates, so drop them. This is wrong. The registered voter count is the denominator for turnout computation. Dropping it before extraction destroys the only turnout signal available at the precinct level.

The correct sequence:

1. **Detect** the row by candidate name pattern (`Registered Voters`, `BALLOTS CAST`, `OVER VOTES`, `UNDER VOTES`, `BLANK`).
2. **Extract** the value into the appropriate field on sibling contest records in the same precinct.
3. **Route** the row to `TurnoutMetadata` contest kind — not `CandidateRace`.
4. **Exclude** the row from candidate-level analysis (margins, competitiveness, entity resolution).

Step 2 is the key. The registered voter count attaches to every contest in the same precinct as a `turnout.registered_voters` field. The ballots cast count becomes `turnout.ballots_cast`. Only after extraction is the metadata row itself reclassified.

## NC SBE Row Format

In the NC SBE precinct results file (`results_pct_20221108.txt`), a registered voter row looks like:

| Column | Value |
|--------|-------|
| Contest Name | `REGISTERED VOTERS - TOTAL` |
| Choice | (empty) |
| Choice Party | (empty) |
| Total Votes | `4,217` |
| Election Day | `4,217` |
| One Stop | `0` |
| Absentee by Mail | `0` |
| Provisional | `0` |

The "Total Votes" column contains the registered voter count, not a vote total. The vote-type breakdown is meaningless (registered voters do not have an election-day vs. early split). L1 extracts `4,217` into `turnout.registered_voters` for precinct P17 in Columbus County, then classifies this row as `TurnoutMetadata`.

The corresponding L1 output:

```json
{
  "contest": {
    "kind": "turnout_metadata",
    "raw_name": "REGISTERED VOTERS - TOTAL"
  },
  "results": [{
    "candidate_name": { "raw": "Registered Voters" },
    "votes_total": 4217
  }],
  "turnout": {
    "registered_voters": 4217
  }
}
```

Sibling contest records in the same precinct (e.g., the school board race) receive:

```json
{
  "turnout": {
    "registered_voters": 4217,
    "ballots_cast": null
  }
}
```

## Scale of the Problem

In the Florida OpenElections dataset, **6,013 rows** are labeled "Registered Voters" — representing **67.9%** of all non-candidate records in that file. Without detection, these rows enter the candidate pipeline as if "Registered Voters" were a person running for office. The L4 LLM audit flagged exactly this pattern in our prototype.

Over Votes and Under Votes are less numerous but equally disruptive. Maine labels its under votes as `BLANK`. Utah includes `TOTAL VOTES` aggregation rows. Each source has its own vocabulary for the same concept.

## Detection Rules

L1 applies pattern matching on the candidate name field before any other processing:

| Pattern | Classification | Action |
|---------|---------------|--------|
| `registered voters` | TurnoutMetadata | Extract to `turnout.registered_voters` |
| `ballots cast` | TurnoutMetadata | Extract to `turnout.ballots_cast` |
| `over ?votes?` | TurnoutMetadata | Extract to `turnout.over_votes` |
| `under ?votes?` | TurnoutMetadata | Extract to `turnout.under_votes` |
| `^blank$` | TurnoutMetadata | Extract to `turnout.under_votes` (ME) |
| `total votes` | TurnoutMetadata | Discard (aggregation artifact) |
| `scattering` | TurnoutMetadata | Extract to `turnout.write_in_scattering` (IA) |

These patterns are checked case-insensitively. They run as the first operation in the L1 pipeline — before name decomposition, before office classification, before FIPS enrichment. A row that matches is routed immediately and never enters the candidate pipeline.