# L4: Canonical — Authoritative Names and Verification

L4 is the final layer. It consumes L3's entity assignments and produces the researcher-facing outputs: canonical names, temporal chains across elections, alias tables, and the results of six verification algorithms. L4 is deterministic given the same L3 input — no LLM calls are made during construction (though the LLM entity audit is part of verification).

## Canonical Name Selection

Each candidate entity has multiple name variants collected from across sources and precincts. L4 selects one canonical name using a fixed algorithm:

1. **Collect all variants.** For entity `person:nc:columbus:lance-timothy-13`, the variants might be `Timothy Lance` (NC SBE), `TIMOTHY LANCE` (MEDSL), and `Lance, Timothy` (OpenElections).

2. **Prefer the most complete.** A variant with a middle initial beats one without. A variant with a suffix beats one without. `SHANNON W BRAY` beats `SHANNON BRAY`. `Robert Williams Jr` beats `Robert Williams` (when they are the same entity — which is rare, since Jr usually indicates a different person).

3. **Among equally complete, prefer the most authoritative source.** Source authority ranking:
   - Certified state data (NC SBE) — highest
   - Academic curated data (MEDSL) — second
   - Community-curated data (OpenElections) — third
   - Election night reporting (Clarity/Scytl) — lowest

4. **Among equally authoritative, prefer the most recent.** A 2022 record beats a 2018 record for the same entity.

The selected canonical name is a presentation choice, not an analytical input. By the time L4 runs, entity resolution is complete — the identity question is settled at L3. L4 is choosing a label for a known entity.

## Temporal Chain Aggregation

L4 builds one temporal chain entry per (entity, election, contest). A candidate who appeared in 47 precincts in one election gets **one** entry with the summed vote total — not 47 entries.

This fixes a prototype bug. The initial implementation built temporal chains per precinct, producing entries like "Timothy Lance, 2022, P17, 303 votes" and "Timothy Lance, 2022, P21, 287 votes." For career tracking and competitiveness analysis, the correct granularity is the election level: "Timothy Lance, 2022, Columbus County Schools Board of Education District 02, 1,531 votes."

The aggregation:

```json
{
  "entity_id": "person:nc:columbus:lance-timothy-13",
  "canonical_name": "Timothy Lance",
  "aliases": ["Timothy Lance", "TIMOTHY LANCE"],
  "elections": [
    {
      "date": "2022-11-08",
      "contest": "Columbus County Schools Board of Education District 02",
      "contest_entity_id": "contest:nc:columbus:school-board-d02",
      "votes": 1531,
      "vote_share": 0.523,
      "outcome": "won",
      "source_count": 1
    }
  ],
  "states": ["NC"],
  "first_appearance": "2022-11-08",
  "election_count": 1
}
```

For multi-cycle candidates, the `elections` array grows. George Dunlap — Mecklenburg County Commissioner across 6 consecutive cycles (2014–2024) — has 6 entries in his temporal chain, each with the contest-level vote total for that election.

## Alias Tables

Every name variant observed for an entity is preserved in the `aliases` array. This serves two purposes:

1. **Searchability.** A user searching for "SHANNON W BRAY" finds the entity whose canonical name is "Shannon W. Bray" because the ALL CAPS variant is in the alias table.

2. **Provenance.** The alias table documents which sources used which name formats. If a future entity resolution decision is questioned, the alias table shows exactly what variants were merged.

Aliases are deduplicated but not normalized — `Timothy Lance` and `TIMOTHY LANCE` are both preserved because they demonstrate that the entity appears in both title-case and all-caps sources.

## The Six Verification Algorithms

L4 runs six verification algorithms over the complete output. These are not optional post-processing — they are integral to the pipeline's trust model. Every verification result is recorded in `verification_report.json`.

### 1. Hash Chain Integrity

Walk the hash chain from L4 → L3 → L2 → L1 → L0 for every record. Recompute each hash and compare to the stored value. Any mismatch identifies the exact layer where the chain breaks.

| Metric | Prototype result |
|--------|:----------------:|
| Records verified | 200 / 200 |
| Broken chains | 0 |
| Layers traversed per record | 5 |

See [Provenance and the Hash Chain](./provenance.md) for the verification algorithm.

### 2. Entity Consistency

Flag entities with characteristics that are unusual for local officeholders:

- **Multi-state entities.** A `candidate_entity_id` spanning NC and FL is suspicious — local officials serve in one state. Federal candidates can span states (a senator's votes appear in statewide and precinct-level records), so federal offices are exempted.
- **Party switches.** An entity appearing as DEM in 2018 and REP in 2022 is not impossible (party switches happen) but is flagged for review.
- **Implausible office combinations.** An entity serving simultaneously as county sheriff and school board member is unlikely (though not impossible in small counties).

### 3. Temporal Plausibility

Check career spans and office progressions:

- **Span check.** An entity with elections in 2006 and 2024 has an 18-year span. Plausible for a long-serving commissioner, but flagged if the office is typically a stepping stone (e.g., school board).
- **Gap detection.** An entity appearing in 2014 and 2024 but not 2016, 2018, 2020, or 2022 may be two different people merged by entity resolution — or someone who left office and returned. Gaps > 2 cycles are flagged.
- **Age plausibility.** If external data (FEC filings, candidate bio pages) provides a birth year, check that the candidate was of legal age at first appearance.

### 4. Cross-Source Reconciliation

Where two sources cover the same contest, compare vote totals for each candidate entity:

| Agreement level | NC 2022 contests | Percentage |
|----------------|:----------------:|:----------:|
| Exact match | 579 | 90.5% |
| Within 1% | 47 | 7.3% |
| Disagree > 1% | 14 | 2.2% |

Disagreements are reported with both sources' totals, the percentage difference, and the probable cause (provisional ballot timing, write-in aggregation, precinct boundary assignment). See [Cross-Source Reconciliation](../hard-problems/cross-source.md).

### 5. Completeness Audit

Report coverage metrics across the full dataset:

| Metric | Target | Prototype result |
|--------|--------|:----------------:|
| State coverage (FIPS populated) | 100% | 100% |
| County coverage (FIPS populated) | 100% | 100% |
| Entity ID fill rate (candidate) | > 95% | 100% |
| Entity ID fill rate (contest) | > 95% | 100% |
| Office classification fill rate | > 90% | 67% (prototype scope) |
| Turnout data fill rate | varies | < 5% (most sources lack it) |

Low fill rates are not errors — they are documented gaps. The completeness audit ensures that gaps are visible, not hidden.

### 6. LLM Entity Audit

For every entity with members from more than one source or more than one election, ask a language model whether the entity cluster is plausible. This is the only LLM call in L4.

The prompt provides the entity's canonical name, all aliases, all elections, all offices, all states, and all vote totals. The model evaluates:

- Is this a plausible single person?
- Are the offices consistent with one career?
- Do the vote totals and geographic spread make sense?
- Are any aliases suspicious (non-person names, ballot measure choices, turnout metadata)?

Prototype results from auditing 50 entities:

| Category | Count | Details |
|----------|:-----:|---------|
| Clean — no issues | 3 | Entity is unambiguous |
| Suspicious — flagged for review | 43 | Precinct-level records inflating temporal chains |
| Likely error — incorrect entity | 4 | "For" and "Against" classified as person entities |

The 43 suspicious entities were a direct consequence of the prototype bug where temporal chains were built per precinct rather than per election. After fixing the aggregation to election-level, the suspicious count dropped to single digits in subsequent runs.

The 4 errors were ballot measure choices ("For", "Against") that had leaked past L1 non-candidate detection and received `candidate_entity_id` values at L3. The LLM audit caught them:

> "'For' is not a plausible person name. This entity appears across 347 contests in 12 states, always in contest names containing 'amendment', 'bond', 'referendum', or 'proposition'. These are ballot measure choices, not candidates."

This finding led to tighter non-candidate detection at L1. See [Non-Candidate Records](../hard-problems/non-candidates.md).

## Output Format

L4 produces three types of output:

### Entity Registries (JSON)

One file per entity type, containing one record per unique entity:

- `candidate_registry.json` — all person entities with canonical names, aliases, temporal chains
- `contest_registry.json` — all contest entities with canonical names, years active, states

### Flat Exports (JSONL and CSV)

One record per candidate per contest per precinct, with canonical names and entity IDs attached:

```json
{
  "election_date": "2022-11-08",
  "state": "NC",
  "county": "COLUMBUS",
  "contest_name": "COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02",
  "candidate_raw": "Timothy Lance",
  "candidate_canonical": "Timothy Lance",
  "candidate_entity_id": "person:nc:columbus:lance-timothy-13",
  "votes_total": 303,
  "source": "nc_sbe",
  "l3_hash": "28183d41d50204d5",
  "l0_hash": "edfedf2760cfd54f"
}
```

The flat export retains precinct-level granularity with entity-level annotations. Users who need contest-level totals aggregate by `(candidate_entity_id, contest_entity_id, election_date)`. Users who need precinct-level data use the records as-is.

The CSV export contains the same fields for users who prefer tabular tools (Excel, R, Stata). Column order matches the JSONL field order.

### Verification Report (JSON)

A single `verification_report.json` summarizing all six verification algorithms:

```json
{
  "run_date": "2026-03-19T12:00:00Z",
  "record_count": 200,
  "entity_count": 206,
  "hash_chain": {"verified": 200, "broken": 0},
  "entity_consistency": {"clean": 195, "flagged": 11},
  "temporal_plausibility": {"clean": 203, "flagged": 3},
  "cross_source": {"exact_match": 579, "within_1pct": 47, "disagree": 14},
  "completeness": {"fips_fill": 1.0, "entity_fill": 1.0, "office_fill": 0.67},
  "llm_audit": {"clean": 3, "suspicious": 43, "error": 4, "entities_audited": 50}
}
```

This report is the pipeline's self-assessment. A researcher evaluating the data reads the verification report first to understand what the pipeline is confident about and where it flagged concerns.

## Cross-References

- [Provenance and the Hash Chain](./provenance.md) — how hash verification works
- [Cross-Source Reconciliation](../hard-problems/cross-source.md) — the NC overlap validation
- [Non-Candidate Records](../hard-problems/non-candidates.md) — the "For" and "Against" audit finding
- [Career Tracking Recipe](../usage/recipe-career-tracking.md) — querying temporal chains
- [Verify a Specific Result](../usage/recipe-verify-result.md) — using hash chains for provenance