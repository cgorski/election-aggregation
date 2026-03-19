# Embedding Model: text-embedding-3-large

The pipeline uses OpenAI's `text-embedding-3-large` for all vector generation at L2. This is a deliberate choice with specific trade-offs. The model is not the best possible embedding model — it is the best available model for this task given current constraints.

## Why text-embedding-3-large

Three properties matter for election entity resolution: dimensionality, consistency, and performance on short structured text.

**3,072 dimensions.** Higher dimensionality preserves more fine-grained distinctions in short strings. "David S Marshall" and "David A Marshall" differ by a single character — a middle initial. In a 384-dimensional space, that distinction may be compressed away. In 3,072 dimensions, the model has room to encode it. We measured: the middle initial drops cosine similarity from 0.7025 to 0.6448 — a 0.058 gap that matters for disambiguation.

**API-based consistency.** Every call to the same model version with the same input produces the same vector. There is no local model initialization, no GPU-dependent floating-point variance, no seed to manage. Two users on different machines embedding the same candidate string get the same 3,072 floats. This is critical for reproducibility: L2 output is deterministic given the same model version.

**Strong on short structured text.** Candidate composite strings are 50–150 characters: `"Timothy Lance | | BOARD OF EDUCATION DISTRICT 02 | NC | Columbus"`. These are not natural language paragraphs — they are structured identifiers with pipe-delimited fields. `text-embedding-3-large` handles this format well in our testing. Nickname pairs (Charlie Crist at 0.451), suffix pairs (Williams Jr at 0.862), and middle-initial pairs (David Marshall at 0.6448) all produce scores in ranges that the cascade can act on.

## Why Not MiniLM

`all-MiniLM-L6-v2` from Sentence Transformers is the default recommendation for lightweight embedding tasks. It runs locally, requires no API key, and produces vectors in milliseconds on CPU. We evaluated it and rejected it for three reasons.

**384 dimensions.** A factor of 8× fewer dimensions than `text-embedding-3-large`. On structured identifiers where single-character differences carry categorical meaning (middle initials, suffixes), the lower dimensionality compresses distinctions. In informal testing, MiniLM scored Williams Jr at 0.91 against Williams — *higher* than `text-embedding-3-large`'s 0.862, and well above any reasonable accept threshold. The suffix signal is effectively lost.

**2021 training data.** MiniLM was trained on data through 2021. It has no exposure to post-2021 candidate names, office titles, or geographic patterns. `text-embedding-3-large` was trained on more recent data, though the exact cutoff is not published. For a task that involves matching strings like "DESANTIS, RON" and "Ron DeSantis" — where the model's familiarity with the name helps — recency matters.

**Weaker on structured identifiers.** MiniLM is optimized for sentence similarity — determining whether two natural language sentences express the same meaning. Our inputs are not sentences. They are pipe-delimited fields with proper nouns, abbreviations, and codes. `text-embedding-3-large` is a general-purpose model that handles structured text more robustly than a sentence-similarity specialist.

MiniLM's advantages — local execution, zero API cost, sub-millisecond inference — are real but irrelevant to our constraints. Budget is not a constraint. Latency at L2 is not a bottleneck (embeddings are computed once and cached). The accuracy difference on structured identifiers is the deciding factor.

## Why Not a Fine-Tuned Model

A model fine-tuned on election name pairs would outperform any general-purpose model. We know this because the failure modes of `text-embedding-3-large` are systematic: it underscores nicknames (Charlie/Charles at 0.451) and overscores suffixes (Williams/Williams Jr at 0.862). A fine-tuned model trained on labeled pairs — "these are the same person" / "these are different people" — would learn that "Jr" is a strong negative signal and that "Charlie"/"Charles" is not.

We do not have training data yet.

Fine-tuning requires labeled pairs: hundreds to thousands of (name_a, name_b, same_person) triples with ground truth. Our prototype has 12 manually verified pairs. The L3 decision log will eventually contain thousands of LLM-confirmed match/no-match decisions — each one a potential training example. This is an active learning loop:

1. L3 uses the general-purpose model to retrieve candidates.
2. The LLM confirms or rejects matches, producing labeled pairs.
3. The labeled pairs train a fine-tuned embedding model.
4. The fine-tuned model replaces `text-embedding-3-large` at L2, improving retrieval.
5. Better retrieval surfaces harder cases for the LLM, producing more informative training data.

This loop is planned but not yet implemented. It requires the pipeline to run at scale first, generating enough decisions for a meaningful training set. In the meantime, `text-embedding-3-large` with the 5-step cascade produces correct results on every tested pair — the LLM compensates for the embedding model's weaknesses.

## Thresholds Are Model-Specific

The calibrated thresholds — auto-accept ≥ 0.95, ambiguous 0.35–0.95, auto-reject < 0.35 — are specific to `text-embedding-3-large` with 3,072 dimensions. A different model produces different similarity distributions. MiniLM's Williams Jr score of 0.91 vs. `text-embedding-3-large`'s 0.862 illustrates the problem: the same pair lands in different threshold zones depending on the model.

If the model changes, recalibration is required:

1. Re-embed all [test cases](../hard-problems/entity-test-cases.md) with the new model.
2. Plot the score distribution for known matches and known non-matches.
3. Find the auto-accept, ambiguous, and auto-reject boundaries that minimize false positives and false negatives.
4. Update the threshold configuration and document the new model in L2 metadata.

The `embedding_model` field stored in every L2 record ensures that thresholds can always be traced to the model that produced the scores. If a record was embedded with `text-embedding-3-large` and the thresholds were calibrated for a hypothetical `election-embed-v1`, the mismatch is detectable.

## Summary

| Property | text-embedding-3-large | MiniLM | Fine-tuned (future) |
|----------|:---------------------:|:------:|:-------------------:|
| Dimensions | 3,072 | 384 | TBD |
| API required | Yes | No | Depends |
| Cost per 1M tokens | ~$0.13 | $0 | $0 (local) |
| Williams Jr score | 0.862 | ~0.91 | Lower (trained) |
| Crist score | 0.451 | ~0.38 | Higher (trained) |
| Training data needed | No | No | Yes (not yet available) |
| Reproducible across machines | Yes | Requires version pinning | Requires version pinning |

The current choice is `text-embedding-3-large` — good enough for the cascade to work, available today, and reproducible without local model management. The long-term path is a fine-tuned model trained on the L3 decision log. The thresholds, the cascade design, and the LLM confirmation step all exist to compensate for the general-purpose model's known weaknesses until that fine-tuned model is ready.