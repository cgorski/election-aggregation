# When the LLM Gets Called (And When It Doesn't)

The LLM is a confirmation tool, not a discovery tool. It is called when cheaper methods have narrowed the problem to a specific, bounded question. It is never called when a deterministic method produces correct results.

This boundary is enforced by pipeline structure, not by discipline. L0 and L1 have no LLM code paths. L2 has none. The LLM is reachable only from L3 (entity resolution and tier 4 office classification) and L4 (entity auditing). A developer cannot accidentally add an LLM call to the parser — the parser runs at L1, which has no API client.

## When the LLM Is Called

Three situations invoke the LLM. Each is a bounded question with structured input and a constrained output format.

### 1. Ambiguous Entity Matches (L3, Step 4)

**Trigger:** Embedding cosine similarity between 0.35 and 0.95 AND the name similarity gate passed (JW on last names ≥ 0.50) AND both candidates are in the same state.

**Input:** Structured name components for both candidates, embedding score, JW score, vote counts, office, state, party.

**Output:** match/no-match, confidence (0.0–1.0), free-text reasoning.

**Model:** Claude Sonnet.

**Volume:** 3.5% of candidate pairs in our prototype (30 calls out of ~850 comparisons). With the step 2.5 gate in place, this drops to near-zero for within-source matching and rises for cross-source matching where name formats diverge.

Real examples:

| Pair | Cosine | LLM Decision | Why LLM was needed |
|------|:------:|:------------:|-------------------|
| Charlie Crist / CRIST, CHARLES JOSEPH | 0.451 | match (0.95) | Nickname below any safe auto-accept threshold |
| Robert Williams / Robert Williams Jr | 0.862 | no match (0.85) | Suffix above old auto-accept; only LLM catches generational distinction |
| Nicole Fried / FRIED, NIKKI | 0.642 | match (0.92) | Nickname in ambiguous zone |

### 2. Tier 4 Office Classification (L2→L3 boundary)

**Trigger:** Office name was not classified by keyword (tier 1), regex (tier 2), or embedding nearest-neighbor with cosine ≥ 0.60 (tier 3).

**Input:** Office name string, state, county, the full taxonomy of (office_level, office_branch) pairs.

**Output:** Classification pair, confidence (0.0–1.0), reasoning.

**Model:** Claude Sonnet.

**Volume:** ~0.5% of unique office names in MEDSL 2022 (~42 of 8,387). By record count, far less — these are the rarest, most obscure offices.

Real examples:

| Office Name | State | LLM Classification | Confidence |
|-------------|-------|-------------------|:----------:|
| Santa Rosa Island Authority | FL | special_district / infrastructure | 0.90 |
| Register of Mesne Conveyances | SC | county / judicial | 0.88 |
| Hog Reeve | NH | municipal / regulatory | 0.60 |

### 3. L4 Entity Auditing

**Trigger:** An entity cluster contains records from multiple sources, multiple elections, or multiple office types. In the current design, every multi-member entity is audited (budget is not a constraint).

**Input:** The full entity cluster — canonical name, all aliases, all elections, all vote counts, all states, all offices.

**Output:** Plausibility assessment: plausible / suspicious / error, with reasoning.

**Model:** Claude Sonnet (Opus-class for flagged entities).

**Volume:** In the prototype, 50 entities were audited. The LLM flagged 43 as suspicious (precinct-level records inflating temporal chains — a bug in our aggregation, not in the data) and 4 as errors ("For" and "Against" classified as person entities). At production scale, the volume scales with the number of multi-member entities, not with total records.

## When the LLM Is Not Called

Everything else. Specifically:

| Operation | Layer | Method | Why not LLM |
|-----------|:-----:|--------|-------------|
| CSV/TSV/XML parsing | L1 | Source-specific parser | Deterministic; format is fixed per source |
| Name decomposition | L1 | Rule-based parser | Deterministic; name formats are enumerable |
| Nickname dictionary lookup | L1 | Hash table | O(1) lookup; no reasoning needed |
| FIPS code enrichment | L1 | Census reference table | Exact match on (state, county_name) |
| Vote share computation | L1 | Arithmetic | Division is deterministic |
| Hash computation | L1–L4 | SHA-256 | Cryptographic function; no reasoning needed |
| Office classification (tiers 1–2) | L1 | Keyword + regex | Deterministic; handles 62% of unique names |
| Office classification (tier 3) | L2 | Embedding nearest-neighbor | Deterministic given model version; handles 4.5% more |
| Embedding generation | L2 | OpenAI API | Deterministic given model version; not an LLM call |
| Exact name matching (step 1) | L3 | Structured field equality | Handles 70% of entity resolution |
| Jaro-Winkler matching (step 2) | L3 | String similarity | Deterministic; handles 0.1% more |
| Name gate (step 2.5) | L3 | JW on last names | Eliminates obvious non-matches |
| High-confidence embedding match (step 3) | L3 | Cosine ≥ 0.95 | Auto-accept; no ambiguity to resolve |
| Canonical name selection | L4 | Fixed algorithm | Most-complete + most-authoritative; no judgment needed |
| Temporal chain aggregation | L4 | Group-by on (entity_id, election_date) | SQL-style aggregation |
| Hash chain verification | L4 | SHA-256 recomputation | Cryptographic verification |
| Cross-source vote reconciliation | L4 | Arithmetic comparison | Exact or percentage-based comparison |

## The Principle

**If a deterministic method handles it, do not add LLM latency and non-determinism.**

This is not a cost argument. Budget is not a constraint. It is an accuracy and reproducibility argument:

1. **Deterministic methods do not hallucinate.** SHA-256 always returns the same hash. FIPS lookup always returns the same code. An LLM might return a different FIPS code on a second call — not because it is wrong, but because it is probabilistic. For operations with known-correct deterministic solutions, adding an LLM is adding risk, not capability.

2. **Deterministic methods are reproducible.** Re-running L1 on the same L0 files with the same parser version produces bit-identical output. Re-running an LLM-based parser may produce different field values. For a pipeline that serves journalists and researchers who need to cite specific numbers, reproducibility is non-negotiable for the operations that support it.

3. **Deterministic methods are fast.** L1 processes 200 records in under a second. An LLM call takes 200–2,000ms. For the 70% of entity resolution handled by exact match and the 62% of office classification handled by keywords, the LLM adds latency with zero accuracy benefit.

The LLM is powerful. It correctly identified all 12 test pairs in entity resolution, including the Crist nickname case (0.451 cosine) that no threshold-based system could safely auto-resolve. It classified all 9 tier-4 office names correctly, including obscure offices like "Hog Reeve" that no reference set could anticipate.

But it is called only for the cases that need it: the 3.5% of entity comparisons in the ambiguous zone, the 0.5% of office names that no pattern matches, and the entity audit that catches contamination like ballot-measure choices misclassified as people. For everything else, the answer is already known — deterministically, reproducibly, and instantly.

## Cross-References

- [Design Principles](./principles.md) — "Deterministic first" as principle #1
- [L3: Matched](./l3-matched.md) — where LLM calls happen for entity resolution
- [The Four-Tier Classifier](../hard-problems/office-four-tiers.md) — where LLM calls happen for office classification
- [Budget Is Not a Constraint](./llm-budget.md) — why the cascade exists despite unlimited budget