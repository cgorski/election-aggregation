# Budget Is Not a Constraint — Speed and Reproducibility Are

This project has no API cost ceiling. Every LLM call that improves accuracy is worth making. This changes several design decisions compared to a cost-constrained pipeline — but it does not change the fundamental architecture. The cascade exists for speed and reproducibility, not for cost savings.

## What Unlimited Budget Changes

### Wider Ambiguous Zone

The embedding similarity thresholds for entity resolution were widened specifically because cost is not a constraint:

| Parameter | Cost-constrained | Our design |
|-----------|:----------------:|:----------:|
| Ambiguous zone | 0.65–0.82 | 0.35–0.95 |
| Zone width | 0.17 | 0.60 |
| Pairs reaching LLM | ~5% of within-block pairs | ~25% of within-block pairs |

The wider zone sends roughly 5× more pairs to the LLM. At $0.0002 per call, the difference between 10,000 calls and 50,000 calls is $8. At production scale with millions of pairs, the difference might reach hundreds of dollars. Neither figure justifies accepting false positives (Williams Jr at 0.862) or false negatives (Crist at 0.451) that a narrower zone would cause.

### Stronger Model for Tiebreakers

Step 5 of the entity resolution cascade escalates low-confidence LLM decisions (confidence < 0.70 from Claude Sonnet) to an Opus-class model. The stronger model costs approximately 10× more per call but is invoked only for the lowest-confidence subset of an already-small LLM cohort.

A cost-constrained pipeline would re-run the same Sonnet model or defer to human review. We use the stronger model because the marginal cost per call (~$0.002) is negligible and the accuracy gain on edge cases is measurable.

### Full L4 Entity Audit

The L4 LLM entity audit examines every multi-member entity — not a sample. In the prototype, 50 entities were audited, catching 43 suspicious records and 4 errors. At production scale with tens of thousands of multi-member entities, full audit coverage means thousands of LLM calls.

A cost-constrained pipeline would sample 5–10% of entities and extrapolate. We audit 100% because the cost of missing a contaminated entity (ballot measure choices classified as people, precinct-level records inflating temporal chains) is higher than the cost of the API calls. The "For" and "Against" error was caught by the full audit — a 10% sample might have missed it.

### Tier 4 Office Classification Without Hesitation

Every unclassified office name that survives tiers 1–3 goes to the LLM. There is no "batch the cheapest 80% and skip the rest" optimization. All ~42 hard cases in our prototype were classified. At national scale, the long tail of hyper-local office names (township-specific roles, water district sub-boards, tribal offices) may produce hundreds of tier 4 calls per election cycle. The cost is trivial; the coverage gain is not.

## What Unlimited Budget Does Not Change

### The Cascade Still Exists

Sending every candidate pair directly to the LLM — skipping exact match, Jaro-Winkler, the name gate, and embedding retrieval — would produce correct results for most pairs. It would also be impossibly slow.

At 42 million rows, even with aggressive blocking, the number of within-block candidate pairs runs into the millions. At 200ms per LLM API call, one million pairs take 55 hours of serial wall-clock time. With 10× parallelism, that is still 5.5 hours — for a single step that exact match handles in seconds for 70% of cases.

The cascade is not a cost optimization. It is a speed optimization. Steps 1–3 process 76% of pairs in under a millisecond each. The LLM is reserved for the 3.5% where cheap methods cannot decide.

### Deterministic Steps Are Still Preferred

Exact match, Jaro-Winkler, keyword classification, regex classification, FIPS lookup, vote share computation, and hash verification are deterministic. They produce identical output from identical input on every run, on every machine, forever.

LLM calls are non-deterministic. The same pair submitted twice may produce different confidence scores (typically within ±0.05) and occasionally different reasoning text. The decision (match/no-match) is stable in >99% of re-runs, but "99% stable" is not "deterministic."

For a pipeline that serves journalists citing specific numbers and researchers publishing reproducible analyses, determinism is not a preference — it is a requirement for the operations that support it. We use deterministic methods wherever they produce correct results, not because they are cheaper, but because they are trustworthy in a way that probabilistic methods are not.

### LLMs Do Not Parse, Enrich, or Compute

No amount of budget makes it sensible to use an LLM for:

- **Parsing CSV/TSV/XML.** The format is fixed per source. A parser handles it in microseconds with zero error rate.
- **FIPS lookup.** A hash table lookup on (state, county_name) returns the correct code every time. An LLM might hallucinate a FIPS code — "37047" for Columbus County NC is correct, but there is no mechanism to verify the LLM's output without the same lookup table that makes the LLM unnecessary.
- **SHA-256 computation.** Cryptographic hash functions are mathematical operations. An LLM cannot compute them.
- **Vote share arithmetic.** 303 / 580 = 0.5224. A calculator is correct. An LLM might round differently, truncate, or occasionally hallucinate.

These operations have known-correct deterministic solutions. Adding an LLM to any of them introduces risk with zero benefit, regardless of budget.

### Reproducibility Requires Logged Decisions

Every LLM decision at L3 and L4 is stored in a JSONL audit log with the full prompt, response, confidence, and reasoning. This is not a cost-saving measure (replay from log avoids re-calling the LLM, saving money). It is a reproducibility measure: a researcher who wants to verify or contest a match decision can read the log, see the LLM's reasoning, and evaluate whether the decision was correct.

If budget were infinite and API calls were instantaneous, we would still log every decision. The log is not a cache — it is the canonical record of how the pipeline resolved ambiguity. Deleting the log and re-running the LLM would produce a slightly different set of confidence scores, which might shift a small number of borderline decisions, which would change downstream entity assignments. The log prevents this drift.

## The Real Constraints

Budget is not a constraint. The real constraints are:

| Constraint | Effect on design |
|------------|-----------------|
| **Wall-clock time** | The cascade exists because LLM calls at scale take hours; exact match takes seconds |
| **Reproducibility** | Deterministic methods preferred; LLM decisions logged for replay |
| **Accuracy** | Wider ambiguous zone, stronger tiebreaker model, full audit coverage |
| **Auditability** | Every decision logged with reasoning; hash chain from L4 to L0 |
| **Correctness** | Deterministic methods used wherever they produce correct results; LLMs used only for genuine ambiguity |

A budget-constrained version of this pipeline would narrow the ambiguous zone, sample the entity audit, skip tier 4 office classification for rare offices, and use the same model for tiebreakers. All of these are accuracy trade-offs. We make none of them.

The cascade's structure — exact match → JW → gate → embedding → LLM → tiebreaker — is identical whether the budget is $10 or $10,000. The thresholds move. The model choices change. The architecture does not.