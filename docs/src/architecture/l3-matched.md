# L3: Matched — Entity Resolution and LLM Confirmation

L3 is the first non-deterministic layer. It resolves entities — determining which records across sources, precincts, and elections refer to the same candidate and the same contest. Every decision is stored in a JSONL audit log with full prompt, response, and reasoning, enabling deterministic replay even though the underlying LLM calls are non-deterministic.

## Input and Output

**Input:** L2 enriched JSONL records with embeddings, composite strings, and quality flags.

**Output:**
- Enriched JSONL with `candidate_entity_id` and `contest_entity_id` assignments.
- A decision log (`candidate_matches.jsonl`) recording every comparison made and its outcome.

## Blocking

Before pairwise comparison begins, records are partitioned into blocks by `(state, office_level, last_name_initial)`. Only pairs within the same block are compared. A candidate for NC school board is never compared to a candidate for FL sheriff.

This reduces the comparison space by approximately four orders of magnitude. The blocking key is deliberately coarse — we accept some noise within blocks (two unrelated people whose last names start with the same letter, in the same state, at the same office level) in exchange for never missing a legitimate match. The step 2.5 gate handles within-block noise cheaply.

## The Five-Step Cascade

| Step | Method | Prototype result | Cost per pair |
|------|--------|:----------------:|:-------------:|
| 1 | Exact match on `(canonical_first, last, suffix)` | 597 (70.0%) | negligible |
| 2 | Jaro-Winkler ≥ 0.92 on full name | 1 (0.1%) | microseconds |
| 2.5 | Name gate: JW on last name < 0.50 → skip | — (gate) | microseconds |
| 3 | Embedding cosine ≥ 0.95 AND same state → auto-accept | 50 (5.9%) | pre-computed |
| 4 | LLM confirmation: cosine 0.35–0.95 | 30 (3.5%) | ~$0.0002/call |
| 5 | Tiebreaker: stronger model when step 4 confidence < 0.70 | 0 (rare) | ~$0.002/call |

Percentages are from the 200-record Columbus County NC prototype. 206 unique candidate entities were created.

### Step 1: Exact Match

The match key is `(canonical_first, last, suffix)` within a `(state, office_level)` block. Timothy Lance appears in 47 precinct rows — all 47 share the same key and collapse to one entity. No fuzzy logic, no API calls.

This step handles the overwhelmingly common case: the same candidate appearing identically across precincts within a single source.

### Step 2: Jaro-Winkler (≥ 0.92)

Catches minor spelling variations that survive L1 parsing — `Mcdonough` vs `McDonough`, transposition errors, inconsistent hyphenation. The threshold of 0.92 is strict to avoid false positives on common surnames.

In the prototype, step 2 resolved 1 additional candidate. Most formatting differences are already normalized at L1.

### Step 2.5: The Name Similarity Gate

Before computing embedding similarity, check last-name Jaro-Winkler. If below 0.50, skip the pair entirely.

This gate was added after a prototype finding. The original cascade had no step 2.5, and all 30 LLM calls were spent on pairs like "Aaron Bridges" vs "Daniel Blanton" — candidates in the same `(NC, school_district, B/D)` block with completely different names. Every call correctly returned no-match, but each cost an API round-trip. The gate eliminates these obvious non-matches before they reach embedding or LLM steps.

At scale, with millions of within-block pairs, this gate prevents orders-of-magnitude waste in downstream steps.

### Step 3: Embedding Auto-Accept (≥ 0.95)

For pairs that pass the gate but did not exact-match, retrieve pre-computed L2 cosine similarity. If ≥ 0.95 AND both candidates are in the same state, auto-accept.

The 0.95 threshold is deliberately high. Robert Williams Jr scored 0.862 against Robert Williams — a false positive under the original 0.82 threshold. At 0.95, only near-identical strings with trivial formatting differences pass. Barbara Sharief at 0.955 is an example that auto-accepts: the only difference is a middle initial `J` added in one source.

A secondary acceptance rule handles the band just below 0.95: embedding ≥ 0.90 AND JW on full name ≥ 0.92 AND same state → accept. This catches Ashley Moody (0.930 cosine) without requiring an LLM call.

### Step 4: LLM Confirmation (0.35–0.95)

Pairs in the ambiguous zone are sent to Claude Sonnet with structured context: both candidates' parsed name components, vote counts, office, state, party, and the embedding score. The LLM returns a decision (match/no-match), confidence (0.0–1.0), and free-text reasoning.

The ambiguous zone is wide (0.35–0.95) by design. Budget is not a constraint. The zone was widened from the original 0.65–0.82 after two findings:
- Charlie Crist at 0.451 — a true match that the old 0.65 reject threshold would have discarded.
- Robert Williams Jr at 0.862 — a false positive that the old 0.82 accept threshold would have merged.

The wider zone sends more pairs to the LLM in exchange for zero threshold-induced errors in the tested range.

### Step 5: Tiebreaker

When step 4 returns confidence below 0.70, the pair escalates to an Opus-class model. This handles unusual nicknames, slight vote-count discrepancies, and geographic ambiguity that Sonnet finds uncertain. Step 5 was not triggered in the 200-record prototype; it exists for the long tail of ambiguity at production scale.

## The Decision Log

Every comparison — not just LLM calls — is recorded in a JSONL audit log at `l3_matched/{state}/{year}/decisions/candidate_matches.jsonl`. One record per pair examined.

An LLM-decided entry:

```json
{
  "decision_id": "a3f8c1d2-4e7b-4a1f-9c3d-8f2e1a6b5c4d",
  "decision_type": "candidate_match",
  "timestamp": "2026-03-19T10:30:00Z",
  "inputs": {
    "name_a": "Charlie Crist",
    "name_b": "CRIST, CHARLES JOSEPH",
    "embedding_score": 0.451,
    "jw_last_name": 1.0,
    "state_a": "FL", "state_b": "FL",
    "contest_a": "Governor", "contest_b": "Governor",
    "votes_a": 3101652, "votes_b": 3101652
  },
  "method": {
    "type": "llm",
    "model": "claude-sonnet-4-20250514",
    "prompt_template_version": "entity_match_v2.0"
  },
  "output": {
    "decision": "match",
    "confidence": 0.95,
    "reasoning": "Charlie is a common nickname for Charles. Same state, same office, identical vote counts."
  }
}
```

An exact-match entry is simpler:

```json
{
  "decision_id": "b7c2e4f1-...",
  "decision_type": "candidate_match",
  "timestamp": "2026-03-19T10:30:01Z",
  "inputs": {
    "name_a": "Timothy Lance",
    "name_b": "Timothy Lance",
    "state_a": "NC", "state_b": "NC"
  },
  "method": {
    "type": "exact",
    "model": null,
    "prompt_template_version": null
  },
  "output": {
    "decision": "match",
    "confidence": 1.0,
    "reasoning": "Exact match on (canonical_first=Timothy, last=Lance, suffix=null)"
  }
}
```

A gate-rejected entry:

```json
{
  "decision_id": "c9d3a5e2-...",
  "decision_type": "candidate_match",
  "timestamp": "2026-03-19T10:30:02Z",
  "inputs": {
    "name_a": "Aaron Bridges",
    "name_b": "Daniel Blanton",
    "jw_last_name": 0.40,
    "state_a": "NC", "state_b": "NC"
  },
  "method": {
    "type": "gate_reject",
    "model": null,
    "prompt_template_version": null
  },
  "output": {
    "decision": "no_match",
    "confidence": 1.0,
    "reasoning": "Last-name JW 0.40 below gate threshold 0.50; skipped."
  }
}
```

## L3 Record Output

Each L1/L2 record is augmented with entity assignments:

```json
{
  "...all L1 and L2 fields...",
  "l3": {
    "l3_hash": "28183d41d50204d5",
    "l2_parent_hash": "854fa6367960bb05",
    "candidate_entity_ids": [
      {"result_index": 0, "entity_id": "person:nc:columbus:lance-timothy-13"}
    ],
    "contest_entity_id": "contest:nc:columbus:school-board-d02"
  }
}
```

The `entity_id` format encodes scope: `person:{state}:{county}:{last}-{first}-{sequence}`. The sequence number disambiguates within a name — necessary when two genuinely different people share the same canonical first and last name in the same county.

Contest entity IDs follow a parallel scheme: `contest:{state}:{county}:{office-slug}`.

## Reproducibility

L3 is non-deterministic because LLM responses may vary between runs. Two strategies make it reproducible in practice:

**Replay from log.** The decision log contains every match decision with its inputs and outputs. Re-running L3 in replay mode reads decisions from the log instead of calling the LLM. This produces identical L3 output — deterministic given the logged decisions.

**Re-run with audit.** Re-running L3 with live LLM calls produces a new decision log. Diffing the two logs reveals any decisions where the LLM changed its mind. In testing, decision stability is high: the same pair with the same context produces the same match/no-match outcome in >99% of re-runs. Confidence scores may vary by ±0.05.

For published results, the decision log is the canonical record. The LLM is a tool that produced the decisions; the decisions themselves are the data.

## The 30 Wasted Calls

The prototype's most actionable finding: all 30 LLM calls were wasted. Every one compared candidates with obviously different names — "Aaron Bridges" vs "Daniel Blanton", "Timothy Lance" vs "Jessica Moore" — that happened to share a blocking key. The embedding scores ranged from 0.55 to 0.73, placing them in the ambiguous zone. The LLM correctly rejected all 30 with high confidence.

The root cause was coarse blocking without a name-similarity pre-filter. The fix — step 2.5, requiring JW ≥ 0.50 on last names before proceeding — would have eliminated all 30 calls. At production scale, this gate is the difference between thousands of useful LLM calls and millions of wasted ones.

## Budget and the Ambiguous Zone

Budget is not a constraint for this project. This changes the threshold calculus:

| Decision | Budget-constrained approach | Our approach |
|----------|----------------------------|-------------|
| Ambiguous zone width | Narrow (0.65–0.82) to minimize LLM calls | Wide (0.35–0.95) to maximize accuracy |
| Step 5 model | Same as step 4 (cheaper) | Opus-class (more capable) |
| Audit coverage | Sample-based | Every multi-member entity audited at L4 |

The wider ambiguous zone means ~25% of within-block pairs reach the LLM, up from ~5% with the old thresholds. The step 2.5 gate keeps the absolute call volume manageable by rejecting pairs with dissimilar last names before they enter the zone.

The cascade still exists despite unlimited budget. Sending every pair to the LLM would take weeks of API calls at 42 million rows — cost is irrelevant when wall-clock time is the bottleneck. And deterministic steps are preferred not because they are cheaper, but because they are reproducible and do not hallucinate.

## Cross-References

- [Entity Resolution Overview](../hard-problems/entity-resolution.md) — the problem and why each step exists
- [The Cascade: Step by Step](../hard-problems/entity-cascade.md) — detailed walkthrough with real examples at every step
- [Real Test Cases](../hard-problems/entity-test-cases.md) — all tested pairs with scores and decisions
- [Threshold Calibration](../hard-problems/entity-thresholds.md) — old vs. new thresholds
- [When the LLM Gets Called](./llm-when.md) — invocation policy across the pipeline
- [Budget Is Not a Constraint](./llm-budget.md) — what unlimited budget changes and what it does not