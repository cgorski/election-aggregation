# Threshold Calibration

Embedding similarity thresholds determine which candidate pairs auto-accept, which enter the LLM zone, and which auto-reject. These thresholds are not universal constants — they are calibrated to a specific embedding model (`text-embedding-3-large`, 3,072 dimensions) using real test data from our prototype.

Two findings from early testing forced a complete recalibration.

## The Two Findings

**Robert Williams Jr at 0.862 — a false positive.** Under the original thresholds, any pair scoring ≥ 0.82 was auto-accepted. "Robert Williams" and "Robert Williams Jr" scored 0.862 — above the threshold. The system would have silently merged father and son into one entity. The suffix "Jr" carries categorical meaning (different person), but the embedding model treats it as a minor token appended to an otherwise identical string.

**Charlie Crist at 0.451 — a false negative.** Under the original thresholds, any pair scoring < 0.65 was auto-rejected. "Charlie Crist" and "CRIST, CHARLES JOSEPH" scored 0.451 — below the threshold. The system would have discarded a true match. The nickname "Charlie" for "Charles", combined with different name ordering, different casing, and an extra middle name, pushed the score well below the reject boundary.

Both errors are unacceptable. Merging different people corrupts every downstream analysis. Missing true matches fragments candidate records across sources, breaking cross-source reconciliation and career tracking.

## Old vs. New Thresholds

| Zone | Old Range | New Range | Change |
|------|-----------|-----------|--------|
| **Auto-accept** | ≥ 0.82 | ≥ 0.95 AND same state | Raised by 0.13, added state constraint |
| **Ambiguous (LLM zone)** | 0.65–0.82 | 0.35–0.95 AND same state | Widened from 0.17 to 0.60 range |
| **Auto-reject** | < 0.65 | < 0.35 OR different state | Lowered by 0.30, added state escape |

The ambiguous zone expanded from a 0.17-wide band to a 0.60-wide band. This means far more pairs are routed to the LLM for confirmation.

## What Each Change Addresses

### Auto-accept raised to 0.95

The Williams Jr pair at 0.862 demonstrated that scores in the 0.82–0.95 range can contain suffix-bearing false positives. At 0.95, the only pairs that auto-accept are near-identical strings with trivial formatting differences — "ASHLEY MOODY" vs "Ashley Moody" (0.930 would not auto-accept; it enters the LLM zone where the model confirms the match using full context).

The same-state constraint is an additional guard. A candidate for county sheriff in Maine should never auto-match with a candidate for county sheriff in Florida, regardless of embedding score. Different-state pairs always enter the LLM zone.

### Ambiguous zone widened to 0.35–0.95

The Crist pair at 0.451 sat in the old auto-reject zone. The new lower bound of 0.35 captures every nickname case we tested:

| Pair | Cosine | Old Zone | New Zone |
|------|:------:|----------|----------|
| DeSantis / DESANTIS, RON | 0.729 | Ambiguous | Ambiguous |
| Crist / CRIST, CHARLES JOSEPH | 0.451 | **Reject** | **Ambiguous** |
| Nicole Fried / FRIED, NIKKI | 0.642 | **Reject** | **Ambiguous** |
| Williams / Williams Jr | 0.862 | **Accept** | **Ambiguous** |
| Val Demings / VAL DEMINGS | 0.828 | Accept | Ambiguous |
| Marco Rubio / RUBIO, MARCO ANTONIO | 0.743 | Ambiguous | Ambiguous |
| Ashley Moody / MOODY, ASHLEY B | 0.930 | Accept | Ambiguous |
| Dale Holness / HOLNESS, DALE V.C. | 0.896 | Accept | Ambiguous |

Under the old thresholds, 3 of 8 pairs were misclassified (2 false accepts, 1 false reject). Under the new thresholds, all 8 enter the LLM zone where the model resolves them correctly.

### Auto-reject lowered to 0.35

Below 0.35, no tested pair in our prototype was a true match. At this score range, the names share almost no surface similarity — they are genuinely different people who happen to share a blocking key.

The different-state escape allows immediate rejection of cross-state pairs regardless of score. Local officeholders do not appear in multiple states. (Federal candidates can, but they are handled by a separate federal-office pathway that does not use this threshold table.)

## The Cost of a Wider Ambiguous Zone

The old ambiguous zone (0.65–0.82) captured roughly 5% of within-block pairs. The new zone (0.35–0.95) captures roughly 25% — a 5× increase in LLM calls.

At prototype scale (200 records), this is negligible. At production scale (42 million rows), the increase matters for throughput but not for budget. Budget is not a constraint. The step 2.5 name gate (JW < 0.50 on last names → skip) eliminates the majority of low-score pairs before they reach the LLM, keeping the actual call volume manageable.

The wider zone is a deliberate trade: more LLM calls in exchange for zero false accepts and zero false rejects in the tested range.

## Thresholds Are Model-Specific

These thresholds are calibrated for `text-embedding-3-large` with 3,072 dimensions. A different model — even an updated version of the same model — will produce different similarity distributions. If the embedding model changes:

1. Re-run the [test cases](./entity-test-cases.md) against the new model.
2. Plot the score distribution for known matches and known non-matches.
3. Recalibrate auto-accept, ambiguous, and auto-reject boundaries.
4. Store the new thresholds alongside the model identifier in L2 metadata.

The `embedding_model` field in every L2 record ensures that thresholds can always be traced to the model that produced the scores.

## Summary

| Principle | Implementation |
|-----------|---------------|
| Never auto-accept a suffix mismatch | Threshold raised to 0.95; suffixes always enter LLM zone |
| Never auto-reject a nickname match | Threshold lowered to 0.35; nicknames always enter LLM zone |
| Cross-state pairs require LLM confirmation | Same-state constraint on auto-accept |
| Wider zone is acceptable | Budget is not a constraint; accuracy is |
| Thresholds are not portable | Model version stored in every record |

## Cross-Reference

- [Real Test Cases](./entity-test-cases.md) — all pairs that informed calibration
- [Suffixes: Jr/Sr Means Different People](./names-suffixes.md) — the Williams Jr finding
- [Nicknames and Middle Initials](./names-nicknames-and-middles.md) — the Crist finding
- [Budget Is Not a Constraint](../architecture/llm-budget.md) — why the wider zone is acceptable