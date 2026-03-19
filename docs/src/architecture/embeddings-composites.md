# Composite String Templates

Embeddings are not generated from raw candidate names. They are generated from composite strings that combine name components with contextual fields — office, state, county, party. This context helps the embedding model distinguish people who share a name but hold different offices in different states. It also introduces a failure mode: context bleed, where shared context artificially inflates similarity between unrelated candidates.

## The Three Templates

Each L2 record generates up to three composite strings, one per embedding type:

| Type | Template | Purpose |
|------|----------|---------|
| Candidate | `{canonical_first} {middle} {last} {suffix} \| {party} \| {office} \| {state} \| {county}` | Entity resolution across sources and elections |
| Contest | `{raw_name} \| {office_level} \| {state} {year}` | Contest entity resolution across naming variants |
| Geography | `{municipality}, {county} County, {state}` | Geographic entity resolution for precinct/place matching |

The pipe character (`|`) is a deliberate separator. It signals to the tokenizer that the fields on either side are distinct semantic units, not a continuous phrase. Without separators, "Timothy Lance DEM" could be tokenized as a three-word name rather than a name followed by a party.

## Real Composite Examples

| Candidate | Composite String |
|-----------|-----------------|
| Timothy Lance (NC, Columbus County school board) | `Timothy  Lance  \| \| BOARD OF EDUCATION DISTRICT 02 \| NC \| Columbus` |
| Charlie Crist (FL, Governor, DEM) | `Charles  Crist  \| DEM \| Governor \| FL \| statewide` |
| CRIST, CHARLES JOSEPH (FL, Governor, DEM) | `Charles Joseph Crist  \| DEM \| Governor \| FL \| statewide` |
| David S Marshall (ME, State Legislature) | `David S Marshall  \| \| State Legislature \| ME \| statewide` |
| David A Marshall (FL, County Commission) | `David A Marshall  \| \| County Commission \| FL \| Broward` |

Note that `canonical_first` is used, not `first`. Charlie Crist's composite uses `Charles` (from the nickname dictionary), not `Charlie`. This means the MEDSL record (`CRIST, CHARLES JOSEPH` → canonical_first `Charles`) and the OpenElections record (`Charlie Crist` → canonical_first `Charles`) produce composites with matching first-name tokens. The remaining divergence — `Joseph` as a middle name — is small enough that the embedding score rises significantly compared to the raw-name embedding.

Empty components produce empty slots. Timothy Lance has no middle name, no suffix, and no party in the NC SBE data. The composite retains the pipe separators with empty fields: `Timothy  Lance  | | BOARD OF EDUCATION DISTRICT 02 | NC | Columbus`. This keeps the template structure consistent across all records, which stabilizes tokenization.

## Why Context Helps: The David Marshall Test

David S. Marshall ran for state legislature in Maine. David A. Marshall ran for county commission in Florida. They are different people. Without context, the embedding model sees two very similar strings.

We measured the effect of context on cosine similarity:

| Composite A | Composite B | Cosine |
|------------|------------|:------:|
| `David Marshall` | `David Marshall` | 1.000 |
| `David Marshall \| ME` | `David Marshall \| FL` | 0.7025 |
| `David S Marshall \| ME` | `David A Marshall \| FL` | 0.6448 |
| `David S Marshall \| \| State Legislature \| ME` | `David A Marshall \| \| County Commission \| FL` | 0.581 |

Each additional contextual field pushes the vectors further apart:

- **State alone** drops similarity from 1.0 to 0.7025. The model encodes `ME` and `FL` as distinct tokens that pull the vectors in different directions.
- **Middle initial** drops it further to 0.6448 — a 0.058 reduction. The single character `S` vs `A` produces measurably different vectors because it changes the token sequence before the separator.
- **Office context** drops it to 0.581. "State Legislature" and "County Commission" are semantically distinct, adding another axis of divergence.

At 0.581, this pair falls well within the ambiguous zone (0.35–0.95) and is routed to the LLM, which correctly rejects the match based on different states, different offices, and different middle initials. Without context, the pair scores 1.0 — an automatic merge of two different people.

The middle-initial contribution (0.058) may seem small, but it matters at the margins. For pairs where state and office are the same — a father and son both serving on the same county commission — the middle initial may be the only signal distinguishing them.

## Why Context Hurts: The Context Bleed Problem

Context is not free. Shared context tokens contribute to vector similarity even when the names themselves are unrelated. This is context bleed.

Consider two candidates in the same NC school district block:

| Candidate | Composite |
|-----------|-----------|
| Aaron Bridges | `Aaron  Bridges  \| \| SCHOOL BOARD \| NC \| Columbus` |
| Daniel Blanton | `Daniel  Blanton  \| \| SCHOOL BOARD \| NC \| Columbus` |

These are completely different people. But their composites share five context tokens: `SCHOOL BOARD`, `NC`, `Columbus`, and the pipe separators. The embedding model encodes these shared tokens into both vectors, producing a cosine similarity of approximately 0.55–0.65 — well above what the names alone would produce (~0.20) and squarely in the ambiguous zone.

In our prototype, all 30 wasted LLM calls were on pairs exactly like this: different people with different names whose shared context inflated their embedding scores into the ambiguous zone. The step 2.5 gate (JW on last names < 0.50 → skip) was added specifically to short-circuit these context-bleed false alarms before they reach the LLM.

### Measuring the bleed

We tested context contribution by varying which fields are included:

| Composite variant | Aaron Bridges vs Daniel Blanton | Cosine |
|-------------------|:-------------------------------:|:------:|
| Name only | `Aaron Bridges` vs `Daniel Blanton` | ~0.21 |
| Name + state | `Aaron Bridges \| NC` vs `Daniel Blanton \| NC` | ~0.38 |
| Name + state + office + county | Full composite | ~0.60 |

Each shared context field adds approximately 0.15–0.20 to the cosine score. For same-name pairs (the cases entity resolution cares about), this boost is helpful — it confirms that two similar names in the same context are likely the same person. For different-name pairs, the same boost is harmful — it inflates scores past the reject threshold.

The step 2.5 gate resolves this asymmetry. If the names themselves are dissimilar (JW < 0.50 on last names), the context-inflated embedding score is irrelevant — the pair is skipped. If the names are similar (JW ≥ 0.50), the context inflation is welcome — it adds corroborating evidence that the similar names in the same context are the same person.

## Design Tradeoffs

### Why not embed names without context?

Bare-name embeddings eliminate context bleed but lose the disambiguation power demonstrated by the David Marshall test. A bare "David Marshall" vs "David Marshall" scores 1.0 — the model cannot distinguish them at all. Context is the only mechanism the embedding model has to separate same-name, different-person pairs.

### Why not use separate embeddings for name and context?

An alternative architecture: embed the name and context separately, then combine scores with weighted averaging. This eliminates context bleed (the name embedding is pure name similarity) while retaining context as a separate signal.

This approach is viable but adds complexity — two embeddings per record instead of one, a tunable weight parameter, and a more complex similarity function. The current single-composite design is simpler and works well with the step 2.5 gate mitigating the primary failure mode. If context bleed proves problematic at scale, split embeddings are a planned fallback.

### Why not fine-tune?

A fine-tuned embedding model trained on election name pairs could learn that `Charlie` and `Charles` are similar, that `Jr` is categorically significant, and that shared context should not inflate scores for dissimilar names. We do not have training data yet.

However, L3 decisions are labeled examples: every LLM match/no-match decision with its confidence and reasoning is a training pair. As the pipeline processes more data, the L3 decision log becomes a natural training set for active learning. A fine-tuned model trained on thousands of L3 decisions would, in principle, learn the domain-specific similarity function that the general-purpose `text-embedding-3-large` approximates. This is a future direction, not a current capability.

## Summary

| Property | Effect | Mitigation |
|----------|--------|------------|
| Context included | Distinguishes same-name, different-person pairs (David Marshall: 1.0 → 0.581) | — (this is the goal) |
| Context bleed | Inflates scores for different-name, same-context pairs (Bridges vs Blanton: 0.21 → 0.60) | Step 2.5 JW gate on last names |
| Middle initial included | Provides disambiguation signal (0.7025 → 0.6448) | — (this is the goal) |
| Nickname dictionary applied | Aligns canonical first names before embedding (Charlie → Charles) | — (this is the goal) |

The composite template is a tradeoff between disambiguation power and noise tolerance. Context helps more than it hurts — but only because the step 2.5 gate exists to catch the cases where it hurts.