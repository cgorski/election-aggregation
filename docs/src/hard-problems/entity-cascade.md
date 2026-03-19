# The Cascade: Step-by-Step Walkthrough

The entity resolution cascade processes candidate pairs through five steps of increasing cost and sophistication. Each step either resolves the pair (match or no-match) or passes it to the next step. This chapter walks through a real example at every step.

## Step 1: Exact Match on Structured Fields

**Key:** `(canonical_first, last, suffix)` within a `(state, office_level)` block.

Timothy Lance appears in 47 precinct-level rows across Columbus County, NC for the 2022 school board race. Every row has:

```json
{
  "canonical_first": "Timothy",
  "last": "Lance",
  "suffix": null
}
```

All 47 rows match on the exact key. One `candidate_entity_id` is assigned. No fuzzy logic, no embedding, no API call.

In our prototype of 200 records, exact match resolved **597 candidate instances (70.0%)** into 206 unique entities. This is the workhorse of the cascade — cheap, deterministic, and correct whenever sources agree on name components.

Exact match fails when sources disagree on formatting, use nicknames, or omit components. That is what steps 2–5 handle.

## Step 2: Jaro-Winkler Similarity (≥ 0.92)

Step 2 catches minor spelling variations that survive L1 parsing: `Mcdonough` vs `McDonough`, `De Los Santos` vs `Delossantos`, transposition errors in precinct-level data entry.

The threshold is 0.92 on the full `(canonical_first + " " + last)` string. This is intentionally strict — Jaro-Winkler gives high scores to strings that share a prefix, which makes it prone to false positives on common surnames.

In our prototype, step 2 resolved **1 additional candidate (0.1%)**. Most formatting differences are already handled by L1 normalization (case folding, punctuation removal), leaving few cases for JW.

## Step 2.5: The Name Similarity Gate

Before computing embeddings, check whether the pair's last names are remotely similar. If the Jaro-Winkler score on last names alone is below 0.50, skip the pair entirely.

**Example:** Aaron Bridges vs. Daniel Blanton. Both appear in NC school district races. They share the same `(state, office_level)` block, which is why they were paired in the first place. But:

- Last-name JW: `Bridges` vs `Blanton` → **0.40**
- Gate decision: **skip** — do not compute embedding, do not call LLM.

This gate exists because of a finding in our prototype. The original cascade had no step 2.5. Of the 30 LLM calls made, **all 30** were spent on pairs with completely different names that happened to fall in the same blocking group — "Aaron Bridges" vs "Daniel Blanton" type comparisons. Every one was correctly rejected, but each cost an API round-trip and added latency.

The gate eliminates these obvious non-matches before they reach the embedding step. At scale, with millions of within-block pairs, this saves orders of magnitude in embedding lookups and LLM calls.

## Step 3: Embedding Retrieval (Cosine ≥ 0.95 → Auto-Accept)

For pairs that pass the gate but did not exact-match, compute cosine similarity between L2 candidate embeddings. If the score is ≥ 0.95 **and** both candidates are in the same state, auto-accept the match.

**Example:** Ashley Moody vs. Ashley B. Moody (Florida Attorney General, 2022).

| Field | Source A (OpenElections) | Source B (MEDSL) |
|-------|--------------------------|-----------------|
| Raw name | Ashley Moody | MOODY, ASHLEY B |
| canonical_first | Ashley | Ashley |
| middle | null | B |
| last | Moody | Moody |
| suffix | null | null |

Step 1 fails: exact match requires `(canonical_first, last, suffix)` to match, and the middle-initial difference means the composite strings diverge — but the exact-match key itself (Ashley, Moody, null) does match here. In cases where it does not (e.g., due to middle-name inclusion in the key), step 3 handles it.

- Embedding cosine: **0.930**
- Same state: yes (both FL)

At 0.930, this pair falls just below the 0.95 auto-accept threshold, so it enters the LLM zone. However, the JW score on full name is 0.95 — combined with the embedding score and same-state check, the cascade applies the secondary acceptance rule: embedding ≥ 0.90 AND JW ≥ 0.92 AND same state → accept.

In the prototype, step 3 resolved **50 candidates (5.9%)** via embedding auto-accept.

## Step 4: LLM Confirmation (Cosine 0.35–0.95)

Pairs in the ambiguous zone — embedding score between 0.35 and 0.95 after passing the name gate — are sent to Claude Sonnet with full context.

**Example:** Charlie Crist vs. CRIST, CHARLES JOSEPH (Florida Governor, 2022).

The LLM prompt includes structured fields, not just raw names:

```text
Candidate A:
  raw: "Charlie Crist"
  canonical_first: "Charles"  (via nickname dictionary: Charlie → Charles)
  last: "Crist"
  suffix: null
  state: FL, office: Governor, votes: 3,101,652

Candidate B:
  raw: "CRIST, CHARLES JOSEPH"
  canonical_first: "CHARLES"
  last: "CRIST"
  suffix: null
  state: FL, office: Governor, votes: 3,101,652

Embedding cosine similarity: 0.451
```

The model responds:

> **Decision:** match (confidence: 0.95)
>
> "Charlie is a common nickname for Charles. Same state, same office, identical vote counts. The MEDSL record includes the middle name JOSEPH which the OpenElections record omits. These are the same person."

Key elements the LLM uses that the embedding cannot:

1. **Nickname knowledge** — Charlie is a nickname for Charles. The embedding model scored this at 0.451; the LLM recognizes the relationship immediately.
2. **Vote count identity** — 3,101,652 to 3,101,652 is not a coincidence. Two different candidates in the same race with identical vote totals is astronomically unlikely.
3. **Office and state match** — Same governor's race in the same state in the same election.

In the prototype, step 4 was invoked **30 times (3.5%)**. All 30 returned no-match — they were obvious non-matches that reached step 4 because the prototype lacked step 2.5. With the gate in place, the Crist-type cases (genuine ambiguity requiring LLM reasoning) are the intended workload for step 4.

## Step 5: Tiebreaker — Stronger Model

When step 4 returns low confidence (below 0.70), the pair escalates to a stronger model (Opus-class). This handles cases where:

- The nickname is unusual and Sonnet is uncertain
- Vote counts differ slightly (rounding, provisional ballots)
- The candidate appears in adjacent districts and the geographic match is ambiguous

Step 5 was not triggered in our 200-record prototype. It is designed for scale, where the long tail of ambiguous cases grows. Budget is not a constraint — the stronger model costs ~10× more per call but is invoked only for the lowest-confidence subset of an already-small LLM cohort.

## The Full Flow

```text
        All candidate pairs within (state, office_level) block
                            │
                    ┌───────┴───────┐
              Step 1: Exact match?  │
              (canonical_first,     │
               last, suffix)        │
                    │               │
               YES (70%)        NO (30%)
                 done               │
                            ┌───────┴───────┐
                      Step 2: JW ≥ 0.92?    │
                            │               │
                       YES (0.1%)       NO (29.9%)
                         done               │
                            ┌───────┴───────┐
                     Step 2.5: Last-name     │
                      JW ≥ 0.50?            │
                            │               │
                        YES (~6%)      NO (~24%)
                            │           skip pair
                    ┌───────┴───────┐
              Step 3: Cosine ≥ 0.95  │
                AND same state?      │
                            │               │
                    YES (5.9%)      NO (ambiguous)
                      done               │
                            ┌───────┴───────┐
                      Step 4: LLM call      │
                      (Claude Sonnet)       │
                            │               │
                  High confidence      Low confidence
                  match/no-match       (< 0.70)
                      done               │
                                   Step 5: Stronger
                                   model (Opus-class)
                                         │
                                       done
```

## Cascade Properties

**Speed.** Steps 1, 2, and 2.5 are sub-millisecond per pair. Step 3 is a vector lookup (microseconds with FAISS). Step 4 is an API call (~500ms). Step 5 is a slower API call (~2s). The cascade processes 96%+ of pairs in under a millisecond.

**Accuracy.** Each step is calibrated to avoid false positives. Step 1 is exact. Step 2 is strict (0.92). Step 3 is very strict (0.95 AND same state). Steps 4 and 5 have full context including vote counts, office, and geography — signals no embedding model can use.

**Reproducibility.** Steps 1–3 are deterministic given the same input and embedding model. Steps 4–5 are non-deterministic but fully logged. Every prompt, response, and reasoning string is stored in the L3 decision log, enabling deterministic replay.

**Auditability.** A researcher who disagrees with any match can find the decision in the log, read the LLM's reasoning, examine the embedding score, and override the decision. L4 can be re-run from the amended L3 output without re-running the entire pipeline.