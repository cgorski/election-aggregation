# Cross-Source Reconciliation

When two independent sources cover the same election, their overlap becomes a validation set. If MEDSL and NC SBE both report results for the same contest in the same county, the vote totals should match. When they do, both sources are credible. When they don't, at least one has an error — and the disagreement reveals data quality issues that no single-source analysis can detect.

North Carolina 2022 is our primary validation case. Both MEDSL and the NC State Board of Elections publish precinct-level results for all NC contests in the 2022 general election.

## The Overlap

We identified **640 contests** present in both MEDSL and NC SBE for the 2022 general election. These span federal, state, county, municipal, judicial, and school board races across all 100 NC counties.

For each contest, we aggregated precinct-level results to the contest level and compared total votes per candidate.

| Agreement Level | Contests | Percentage |
|----------------|:--------:|:----------:|
| Exact vote total match | 579 | 90.5% |
| Within 1% of each other | 47 | 7.3% |
| Disagree by more than 1% | 14 | 2.2% |
| **Total** | **640** | **100%** |

**90.5% exact match** across 640 contests, derived from two completely independent data pipelines (MIT's academic processing vs. NC's official state board reporting), is strong evidence that both sources are faithfully representing the same underlying certified results.

## The 7.3% — Small Disagreements

The 47 contests with near-matches (within 1%) trace to identifiable causes:

| Cause | Contests | Notes |
|-------|:--------:|-------|
| Provisional ballot inclusion timing | 22 | MEDSL snapshot taken before final canvass; NC SBE includes provisionals |
| Precinct boundary rounding | 11 | Split precincts assigned differently by each source |
| Write-in aggregation | 9 | NC SBE reports individual write-ins; MEDSL aggregates to "Write-in" |
| Unknown | 5 | Under investigation |

These are not errors — they are legitimate differences in how two organizations process the same raw certified results. Provisional ballot timing is the most common cause: MEDSL's data may reflect an earlier snapshot of the canvass than NC SBE's final certified totals.

## The 2.2% — Real Disagreements

The 14 contests with >1% disagreement require individual investigation. Common causes include:

- **Misassigned precincts.** A precinct's results attributed to the wrong contest or district in one source.
- **Partial data.** One source missing results from a subset of precincts, typically in multi-county contests where one county's data arrived late.
- **Candidate name mismatch causing split.** The same candidate's votes split across two entity IDs in one source because a name variant was not resolved — e.g., "JOHN SMITH" in early voting vs. "John R. Smith" in election-day results treated as different candidates.

These 14 cases are flagged by the L4 cross-source reconciliation algorithm and reported in the verification output. They are not silently ignored.

## Name Formatting Differences

Vote totals may agree, but candidate names almost never do. Of the 640 overlapping contests, **401 (62.7%)** have at least one candidate whose name is formatted differently between MEDSL and NC SBE.

| Formatting Difference | Example (MEDSL) | Example (NC SBE) | Frequency |
|----------------------|-----------------|-------------------|:---------:|
| ALL CAPS vs Title Case | `TIMOTHY LANCE` | `Timothy Lance` | 389 |
| Last-first vs first-last | `LANCE, TIMOTHY` | `Timothy Lance` | 247 |
| Middle initial present/absent | `SHANNON W BRAY` | `Shannon W. Bray` | 118 |
| Period after middle initial | `SHANNON W BRAY` | `Shannon W. Bray` | 94 |
| Nickname in quotes vs parens | `CHARLES "CHARLIE" CRIST` | `Charles (Charlie) Crist` | 12 |
| Suffix formatting | `ROBERT WILLIAMS JR` | `Robert Williams, Jr.` | 31 |
| Prefix/title included | `HON. JANE DOE` | `Jane Doe` | 8 |

A single candidate can exhibit multiple formatting differences simultaneously. "BRAY, SHANNON W" (MEDSL) vs "Shannon W. Bray" (NC SBE) combines casing, ordering, and punctuation differences in one pair.

This is why entity resolution exists. The vote totals confirm these are the same contests with the same candidates. The name formatting confirms that string equality is insufficient — structured decomposition, embedding, and in some cases LLM confirmation are required to link records across sources.

## This Overlap as a Validation Set

The 640-contest NC overlap serves three purposes in the pipeline:

### 1. Entity Resolution Validation

For every candidate pair that the L3 cascade matches across MEDSL and NC SBE, we can verify the match by comparing vote totals. If the cascade says "TIMOTHY LANCE" (MEDSL) and "Timothy Lance" (NC SBE) are the same person, and their vote totals match exactly, the match is confirmed by an independent signal. If the cascade says they match but the vote totals disagree by 50%, the match is suspect.

### 2. Office Classification Validation

Both sources cover the same contests but use different office name strings. MEDSL might report "NC HOUSE OF REPRESENTATIVES DISTRICT 047" while NC SBE reports "NC HOUSE OF REPRESENTATIVES - DISTRICT 47". If both classify to `state/legislative`, the classifier is consistent. If one classifies to `state/legislative` and the other to `county/legislative`, we have a bug.

### 3. Parser Validation

When two independent parsers (the MEDSL parser and the NC SBE parser) produce the same vote counts for the same contest, both parsers are likely correct. When they disagree, the disagreement localizes the bug to one parser or the other — far easier to debug than a single-source pipeline where errors are invisible.

## Beyond NC

The NC overlap is our deepest validation case because NC SBE publishes granular, machine-readable precinct data going back to 2006. Other states offer less overlap:

| State | MEDSL 2022 | Secondary Source | Overlap Quality |
|-------|:----------:|-----------------|:---------------:|
| NC | Yes | NC SBE (precinct-level, 2006–2024) | High |
| FL | Yes | OpenElections (county-level, select years) | Medium |
| OH | Yes | OpenElections (precinct-level, 2022) | Medium |
| GA | Yes | Clarity/Scytl (election night, unstable URLs) | Low |
| All others | Yes | MEDSL only | None |

As additional state-level sources are integrated, each creates a new validation pair. The architecture is designed to scale: the L4 cross-source reconciliation algorithm runs for any pair of sources that cover the same (state, year, contest) combination. No code changes are required — only new L0 data and a new L1 parser.

## The Lesson

Cross-source reconciliation is not a feature — it is the only reliable way to detect errors in election data. A single source can be internally consistent and still wrong. Two independent sources that agree are almost certainly right. Two independent sources that disagree tell you exactly where to look.

The 90.5% exact match rate across 640 NC contests is our current evidence floor. Every additional source and state that achieves similar agreement raises confidence in the pipeline. Every disagreement is a bug report — either in our pipeline or in the source data.