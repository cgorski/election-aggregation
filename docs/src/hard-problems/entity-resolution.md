# Entity Resolution

Entity resolution — determining that two records refer to the same human being — is the single hardest problem in this project. It is also the most consequential. Eight of the 30 query types identified in [What Questions Should Be Answerable](../problem/questions.md) depend on it: career tracking, cross-source reconciliation, candidate deduplication, party switch detection, multi-cycle competitiveness analysis, incumbent identification, name standardization, and cross-election turnout comparison.

The problem is cross-cutting. It touches every source, every state, every election, and every office level. Get it wrong and you merge fathers with sons, split one candidate into three, or silently drop a career that spans six election cycles.

## The Scale Problem

MEDSL 2022 alone contains approximately 42 million rows. A naive all-pairs comparison would require ~8.8 × 10¹⁴ similarity computations. Even at 1 million comparisons per second, that is 28 years of wall-clock time. Entity resolution at this scale requires a cascade that eliminates the vast majority of comparisons before reaching expensive methods.

## The Cascade

Our entity resolution pipeline is a five-step cascade. Each step is cheaper and faster than the next. Each step either resolves the pair (match or no-match) or passes it to the next step.

| Step | Method | Resolves | Cost per pair |
|------|--------|:--------:|:-------------:|
| 1 | Exact match on `(canonical_first, last, suffix)` | 70.0% | negligible |
| 2 | Jaro-Winkler similarity ≥ 0.92 | 0.1% | microseconds |
| 2.5 | Name similarity gate: JW on last name < 0.50 → skip | — | microseconds |
| 3 | Embedding cosine similarity ≥ 0.95 → auto-accept | 5.9% | pre-computed |
| 4 | LLM confirmation (cosine 0.35–0.95) | 3.5% | ~$0.0002 |
| 5 | Tiebreaker: stronger model | rare | ~$0.002 |

Pairs that are not resolved by step 5 are escalated to human review.

## Prototype Results

Our prototype processed 200 NC SBE records from Columbus County, NC (2022 general election):

| Metric | Value |
|--------|-------|
| Input records | 200 |
| Exact matches (step 1) | 597 (70.0%) |
| Jaro-Winkler matches (step 2) | 1 (0.1%) |
| Embedding auto-accepts (step 3) | 50 (5.9%) |
| LLM calls (step 4) | 30 (3.5%) |
| LLM matches confirmed | 0 |
| LLM no-matches confirmed | 30 |
| Unique candidate entities created | 206 |
| Hash chains verified | 200/200 |

All 30 LLM calls were spent on pairs that shared a blocking key (same state, same office level, same last-name initial) but had completely different names — comparisons like "Aaron Bridges" vs "Daniel Blanton" that happened to fall within the same block. Every one was correctly rejected. This finding led to step 2.5: the Jaro-Winkler gate on last names. If the JW score on last names alone is below 0.50, skip the pair entirely. This would have eliminated all 30 wasted LLM calls.

## Why Embedding Alone Fails

Embedding similarity is a powerful retrieval signal but an unreliable decision signal. Two real cases demonstrate the failure modes:

**False negative — Charlie Crist at 0.451.** MEDSL records `CRIST, CHARLES JOSEPH`. OpenElections records `Charlie Crist`. The embedding model scores their cosine similarity at 0.451. Any threshold-based system that relies solely on embeddings either rejects this pair (missing a true match) or sets the accept threshold so low that it admits thousands of false positives.

The problem is structural. The embedding model sees different surface forms — different name ordering, different casing, a nickname versus a legal name, and a middle name present in one source but not the other. The model has no reliable mechanism to know that Charlie is a common nickname for Charles.

**False positive — Robert Williams Jr at 0.862.** `Robert Williams` and `Robert Williams Jr` score 0.862. The model treats "Jr" as a minor token appended to an otherwise identical string. But Jr is a generational suffix — these are different people. At our original auto-accept threshold of 0.82, this pair would have been silently merged.

The embedding model is good at detecting surface similarity. It is bad at understanding that a single token ("Jr") carries categorical meaning, and that a short nickname ("Charlie") maps to a longer legal name ("Charles").

## Why LLM Alone Fails

An LLM like Claude Sonnet can correctly resolve both cases above. It knows Charlie is a nickname for Charles. It knows Jr indicates a different person. In our tests, it correctly identified all 11 test pairs with appropriate confidence levels.

But LLM-only resolution is infeasible at scale:

- **Speed:** At 200ms per API call, resolving 42 million pairwise comparisons would take years. Even with aggressive blocking, the number of candidate pairs runs into millions.
- **Reproducibility:** LLM outputs are non-deterministic. Running the same pair twice may produce different confidence scores. This is acceptable for ambiguous cases but wasteful for the 70% of cases that exact match handles perfectly.
- **Cost:** While budget is not a constraint, sending millions of obvious matches and obvious non-matches to an LLM is pure waste. The LLM adds value only on the ambiguous cases that simpler methods cannot resolve.

## Why the Cascade Works

The cascade combines the strengths of each method:

1. **Exact match** handles the common case (70%) — same name, same state, different precincts. No ML, no API calls, no latency, no non-determinism.

2. **Jaro-Winkler** catches minor spelling variations ("SHANNON W BRAY" vs "Shannon W. Bray") that exact match misses due to casing or punctuation. Still deterministic, still free.

3. **The name gate** (step 2.5) eliminates pairs that share a blocking key but have obviously different names. This prevents the "wasted 30 LLM calls" scenario from the prototype. Deterministic, zero cost.

4. **Embedding retrieval** identifies high-confidence matches (≥ 0.95) where the names differ in format but not in substance. Pre-computed vectors make this effectively free at query time. The 0.95 threshold is deliberately conservative — only near-certain matches pass.

5. **LLM confirmation** handles the hard cases: nicknames (Crist at 0.451), suffixes (Williams Jr at 0.862), ambiguous common names. The LLM sees structured name components, vote counts, office, state, and party — enough context to reason about identity. Every prompt, response, and reasoning chain is stored for audit.

6. **Tiebreaker** (step 5) escalates low-confidence LLM decisions to a stronger model (Opus-class). This adds cost but catches cases where Sonnet is uncertain.

The cascade balances three properties:

- **Accuracy:** The LLM catches what embeddings miss. Embeddings retrieve what exact match misses. Each layer covers the failure modes of the layer above it.
- **Speed:** 70% of resolution is free. 6% is pre-computed. Only 3.5% requires API calls. At scale, this is the difference between hours and years.
- **Reproducibility:** Steps 1–3 are fully deterministic. Steps 4–5 are non-deterministic but logged — every decision can be replayed from the audit log without re-invoking the LLM.

## The 19 Exact Ties

Entity resolution is a prerequisite for detecting exact ties. In MEDSL 2022, we found 19 contests nationally where the top two candidates received exactly the same number of votes. Without entity resolution, precinct-level records cannot be aggregated into contest-level totals — and ties cannot be detected.

## Blocking Strategy

Before the cascade runs, records are partitioned into blocks by `(state, office_level, last_name_initial)`. Only pairs within the same block are compared. This reduces the comparison space by approximately four orders of magnitude while preserving all legitimate matches — a candidate for NC school board is never compared to a candidate for FL sheriff.

The blocking key is deliberately coarse. We accept some wasted comparisons within blocks (like "Aaron Bridges" vs "Daniel Blanton" in the same NC school_district block) in exchange for never missing a legitimate match. The step 2.5 gate handles the within-block noise.

## Detailed Walkthroughs

- [The Cascade: Step by Step](./entity-cascade.md) — each step with real examples
- [Real Test Cases from Real Data](./entity-test-cases.md) — all tested pairs with scores and decisions
- [Threshold Calibration](./entity-thresholds.md) — how Williams Jr and Crist changed the thresholds