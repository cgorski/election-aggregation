# Contest Disambiguation

Three distinct problems hide under one label: the same office name can mean different races, the same race can have different names, and some races elect multiple winners. Each breaks a different assumption in the pipeline.

## Problem 1: Same Office Name, Different Races

Harris County, Texas elects 25 district court judges. Every one of them appears in the data as `DISTRICT COURT JUDGE`. Without the `district` column, all 25 races collapse into a single contest — 25 winners, 50+ candidates, and no way to compute margins or determine who ran against whom.

The distinguishing field varies by source:

| Source | Office name | Distinguishing field | Example value |
|--------|------------|---------------------|---------------|
| MEDSL | `DISTRICT COURT JUDGE` | `district` | `127TH` |
| NC SBE | `DISTRICT COURT JUDGE DISTRICT 13B SEAT 02` | Embedded in contest name | `13B SEAT 02` |
| OpenElections | `District Court Judge` | Separate `district` column | `127` |

MEDSL separates the seat identifier into a dedicated column. NC SBE concatenates it into the contest name string. OpenElections does both, inconsistently, depending on the state contributor.

The L1 parser must extract the seat identifier regardless of where it appears. The contest entity key is `(state, county, office_name, district, seat)` — not just `(state, county, office_name)`. Omitting `district` or `seat` merges distinct races.

Real examples from MEDSL 2022:

| State | Office name | Distinct seats | What disambiguates |
|-------|------------|:--------------:|-------------------|
| TX | DISTRICT COURT JUDGE | 25 | `district` column: 11TH, 55TH, 80TH, ... |
| NC | DISTRICT COURT JUDGE | 48 | Contest name suffix: `DISTRICT 13B SEAT 02` |
| OH | COURT OF COMMON PLEAS | 14 | `district` column: GENERAL DIVISION, DOMESTIC |
| FL | COUNTY COURT JUDGE | 6–12 per county | `district` column: GROUP 1, GROUP 2, ... |

Florida's `GROUP` numbering is particularly treacherous. "COUNTY COURT JUDGE GROUP 3" in Miami-Dade is a different contest from "COUNTY COURT JUDGE GROUP 3" in Broward. The county is part of the disambiguation key.

## Problem 2: Same Race, Different Names Across Years

NC SBE data from 2014 labels a state house seat as `NC HOUSE OF REPRESENTATIVES DISTRICT 03`. In 2018, redistricting renamed it to `NC HOUSE OF REPRESENTATIVES DISTRICT 3`. In 2022, the same source uses `DISTRICT THREE` in some contest types.

All three strings refer to the same legislative seat. But to a string-matching system, they are three different contests. Tracking a candidate's career across elections requires knowing that `DISTRICT 03`, `DISTRICT 3`, and `DISTRICT THREE` are the same district.

Common variations found in NC SBE data:

| Variant A | Variant B | Variant C | Same contest? |
|-----------|-----------|-----------|:-------------:|
| DISTRICT 03 | DISTRICT 3 | DISTRICT THREE | Yes |
| BOARD OF EDUCATION | BD OF ED | BOE | Yes |
| COUNTY COMMISSIONERS | COUNTY COMMISSION | BOARD OF COMMISSIONERS | Yes |

This is contest entity resolution — the same problem as candidate entity resolution, applied to office names instead of person names. The cascade applies:

1. **Normalize numbers**: Strip leading zeros, convert written numbers to digits. `DISTRICT 03` → `DISTRICT 3`, `DISTRICT THREE` → `DISTRICT 3`.
2. **Abbreviation expansion**: `BD OF ED` → `BOARD OF EDUCATION`, `COMM` → `COMMISSION`.
3. **Embedding similarity**: For remaining ambiguous pairs, compute cosine similarity on contest composite strings and apply the same threshold logic as candidate matching.

Contest entity resolution runs at L3 alongside candidate entity resolution. Each contest receives a `contest_entity_id` that persists across election cycles.

## Problem 3: Multi-Seat Contests

A "vote for 3" school board race elects the top three candidates. The standard margin computation — difference between first and second place — does not apply. The meaningful margin is between the last winner (3rd place) and the first loser (4th place).

The `vote_for` field (called `magnitude` in some sources) records how many seats are being filled. MEDSL provides this field for most contests. NC SBE does not — it must be inferred from ballot instructions embedded in the contest name or from the number of candidates who received non-trivial vote shares.

Real example from Dawson County, Georgia (2022):

| Contest | vote_for | Candidates | Votes |
|---------|:--------:|:----------:|-------|
| Board of Education | 3 | 6 | 25,186 / 25,186 / 24,901 / 24,844 / 23,112 / 22,987 |

The effective margin is between 3rd place (24,901) and 4th place (24,844) — a gap of 57 votes. Reporting the margin as the gap between 1st and 2nd (0 votes — an exact tie) is misleading: the tie is between the top two winners, not between a winner and a loser.

Worse, the exact tie at the top (25,186 each) may trigger recount rules in some jurisdictions. Whether a recount applies depends on whether the tied candidates are competing for the same seat or are both safely elected. The `vote_for` field is the only way to know.

### Why `vote_for` matters for competitiveness analysis

Without `vote_for`, every multi-seat contest looks either wildly competitive (if you compare 1st to 2nd among co-winners) or wildly uncompetitive (if you compare any winner to any loser in a field of 6). The correct margin — last winner vs. first loser — requires knowing the cutoff.

| Analysis | Without `vote_for` | With `vote_for` |
|----------|-------------------|----------------|
| Is the race competitive? | Unclear — 0-vote "margin" is misleading | Margin of 57 votes at the cutoff |
| Is it uncontested? | 6 candidates — looks contested | Only if ≤ 3 candidates filed |
| Who won? | Top 1? Top 2? Unknown | Top 3 |

### Detection when the field is missing

When `vote_for` is absent (NC SBE, some OpenElections files), L1 applies heuristics:

1. **Contest name pattern**: "VOTE FOR 3", "ELECT 2", "(3 SEATS)" embedded in the contest name string.
2. **Candidate count**: If 6+ candidates appear in a school board or city council race, flag for multi-seat review.
3. **Vote distribution**: If the top N candidates have similar vote totals and a clear drop-off to N+1, infer N seats.

These heuristics are imperfect. The `vote_for` field, when present, overrides all heuristics. When absent, the inferred value is stored with a confidence flag, and the L4 verification audit reviews flagged contests.

## How All Three Interact

A single contest can exhibit all three problems simultaneously. Consider a Texas county with five JP (Justice of the Peace) precincts, each electing one JP, across three election cycles where the contest name changed from "J.P. PCT 3" to "JUSTICE OF THE PEACE PRECINCT 3" to "JP PRECINCT THREE":

- **Problem 1**: Five precincts, five separate contests, all labeled variants of "Justice of the Peace".
- **Problem 2**: Three different name formats across 2018, 2020, 2022 for each precinct.
- **Problem 3**: Each is single-seat, but a neighboring school board race on the same ballot elects three members.

The contest entity key `(state, county, office_name_normalized, district_normalized, seat)` disambiguates problem 1. Contest entity resolution across years handles problem 2. The `vote_for` field handles problem 3. All three solutions must work together for the contest record to be correct.