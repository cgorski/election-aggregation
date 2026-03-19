# Suffixes: Jr/Sr Means Different People

A suffix is not decoration. Robert Williams and Robert Williams Jr. are two different human beings — father and son, or grandfather and grandson. Any system that strips suffixes before matching will merge them into one entity.

This is not a hypothetical risk. It happened in our prototype.

## The Williams Problem

In Florida 2022, two candidates share the name Robert Williams:

| Source | Raw Name | Office | Votes |
|--------|----------|--------|---------|
| MEDSL | ROBERT WILLIAMS | State House District 41 | 12,304 |
| OpenElections | Robert Williams Jr | State House District 41 | 12,304 |

The embedding similarity between these two strings is **0.862** — well above our original auto-accept threshold of 0.82. Without suffix-aware logic, the cascade would have merged them automatically, no LLM consulted.

The LLM caught it. Given the full context, Claude Sonnet returned:

> **Decision:** no_match (confidence: 0.85)
>
> "The suffix 'Jr' indicates a generational distinction. 'Robert Williams' and 'Robert Williams Jr' are most likely two different individuals — typically father and son. While the office and vote counts match, the suffix is a strong signal that these are distinct people. The MEDSL record may have dropped the suffix during their cleaning process. I recommend treating these as separate entities until a human reviewer can confirm."

This single case changed our threshold design.

## How This Changed the Thresholds

Before Williams Jr., auto-accept was ≥ 0.82. A score of 0.862 would have been accepted without review. After this finding, we raised auto-accept to **≥ 0.95**, ensuring that suffix-bearing pairs always enter the LLM zone (0.35–0.95) where the model can reason about generational distinctions.

| Threshold | Before | After | Reason |
|-----------|--------|-------|--------|
| Auto-accept | ≥ 0.82 | ≥ 0.95 | Williams Jr at 0.862 was a false positive |
| Ambiguous (LLM zone) | 0.65–0.82 | 0.35–0.95 | Wider zone catches more edge cases |
| Auto-reject | < 0.65 | < 0.35 | Crist at 0.451 was a false negative |

The wider ambiguous zone sends more pairs to the LLM. Budget is not a constraint — accuracy is.

## Suffix-Aware Logic in the Cascade

Suffixes receive special treatment at multiple stages:

**L1 — Decomposition.** The name parser extracts Jr, Sr, II, III, IV, V, Esq, and PhD into the `suffix` field. Both `Jr.` and `Jr` (with and without period) normalize to `Jr`. The suffix is never discarded.

**Step 1 — Exact match.** The exact match key is `(canonical_first, last, suffix)`. "Timothy Lance" and "Timothy Lance" match. "Robert Williams" and "Robert Williams Jr" do not — the suffix field differs (null vs "Jr").

**Step 3 — Embedding.** The suffix is included in the composite string: `{canonical_first} {middle} {last} {suffix} | {party} | {office} | {state} | {county}`. This means "Robert Williams" and "Robert Williams Jr" produce different vectors, but the difference is small (0.862 cosine) because the model treats "Jr" as a minor token.

**Step 4 — LLM confirmation.** The prompt explicitly includes both suffix fields and instructs the model: "A suffix like Jr or Sr typically indicates a different person (parent vs child). Do not match across suffixes unless you have strong evidence they refer to the same individual." The LLM sees the structured fields, not just the raw strings.

## The Suffix Inventory

From MEDSL 2022 data across all 50 states:

| Suffix | Occurrences | Notes |
|--------|-------------|-------|
| Jr | 1,847 | Most common; often dropped by one source |
| Sr | 312 | Almost always appears alongside a Jr in the same jurisdiction |
| II | 478 | Increasingly common; same disambiguation need as Jr |
| III | 189 | Rarer but unambiguous signal |
| IV | 31 | |
| V | 4 | |

The Jr/Sr problem is not rare. Nearly 2,000 candidates in a single election cycle carry a Jr suffix, and an unknown number of their non-suffixed counterparts exist in the same dataset.

## When Suffixes Are Missing

The harder case is when one source includes the suffix and another drops it. MEDSL strips suffixes more aggressively than NC SBE. OpenElections preserves them inconsistently. This means the cascade must handle the asymmetric case: one record has suffix "Jr", the other has suffix null.

The rule: **a null suffix does not match a non-null suffix.** Null-to-null matches normally. "Jr" to "Jr" matches normally. But null to "Jr" always enters the LLM zone, regardless of embedding score. The LLM can then examine vote counts, office, and geographic context to determine whether the missing suffix is a data quality issue (same person, suffix dropped) or a genuine distinction (father and son).

This is conservative by design. We would rather send 1,847 extra pairs to the LLM than silently merge fathers with sons.

## Cross-Reference

- [Entity Resolution Overview](./entity-resolution.md) — where suffixes fit in the cascade
- [Threshold Calibration](./entity-thresholds.md) — old vs. new thresholds driven by this finding
- [Real Test Cases](./entity-test-cases.md) — Williams Jr and all other tested pairs