# Why the Order Matters: Clean → Embed → Match → Canonicalize

The pipeline's four processing stages must run in exactly this order. This is not a convention — it is a dependency chain where each stage requires the output of all prior stages. Rearranging them destroys signal.

We learned this the hard way.

## The Insight

The original prototype ran normalization aggressively: strip middle initials, collapse suffixes, force uppercase, pick a canonical name, *then* try to match entities. The sequence was:

```text
Old order:  Canonicalize → Match
            (normalize aggressively, then find duplicates)
```

This destroyed the information needed to tell different people apart.

**David S. Marshall** (Maine, state legislature) and **David A. Marshall** (Florida, county commission) are two different people. Under the old pipeline, both names were normalized to `MARSHALL, DAVID` — middle initials stripped as noise. After normalization, the two records were indistinguishable. The entity resolver matched them as the same person. One David Marshall absorbed the other's career, vote history, and geographic record.

The embedding scores confirm why middle initials matter:

| Composite string | Cosine similarity |
|-----------------|:-----------------:|
| `David Marshall | ME` vs `David Marshall | FL` (no middle) | 0.7025 |
| `David S Marshall | ME` vs `David A Marshall | FL` (with middle) | 0.6448 |

The middle initial drops the score by 0.058 — enough to push the pair further from the accept threshold and toward correct rejection. But this signal only exists if the middle initial survives to L2. If L1 strips it during "normalization," it is gone forever.

## The Correct Order

```text
L1  CLEAN        Parse into components. Preserve everything:
                 first, middle, last, suffix, nickname, canonical_first.
                 No components are discarded. No names are collapsed.
 ↓
L2  EMBED        Generate vectors from composite strings that include
                 middle initials, suffixes, and canonical_first.
                 The embedding encodes all preserved signal.
 ↓
L3  MATCH        Compare embeddings. Run LLM confirmation on ambiguous
                 pairs. The LLM sees structured components — middle
                 initials, suffixes, nicknames — and reasons about them.
 ↓
L4  CANONICALIZE Now that entities are resolved, pick the authoritative
                 name. Prefer the most complete variant. Build alias
                 tables. Aggregate temporal chains.
```

Each stage depends on prior stages' output:

- **L2 depends on L1** — embeddings are generated from L1's structured name components. If L1 strips middle initials, L2 cannot encode them.
- **L3 depends on L2** — entity resolution uses L2 embeddings as the retrieval step. If L2 has degraded vectors (because L1 destroyed signal), L3 makes worse decisions.
- **L4 depends on L3** — canonical name selection requires knowing who the person is. You cannot pick the "best" name for an entity before you know which records belong to that entity.

## What Breaks If You Rearrange

### Canonicalize before Match

This is the old pipeline. Normalize aggressively, then match. Failures:

- `David S. Marshall` and `David A. Marshall` merge into one entity.
- `Robert Williams` and `Robert Williams Jr` merge — suffix stripped before matching can use it.
- `Charlie Crist` normalizes to `CRIST, CHARLIE` but `CRIST, CHARLES JOSEPH` normalizes to `CRIST, CHARLES` — the canonical forms *don't match*, so the same person splits into two entities.

Aggressive normalization both merges people who should be separate and splits people who should be merged. It is wrong in both directions simultaneously.

### Match before Embed

Without embeddings, matching falls back to string similarity alone. Jaro-Winkler on `Charlie Crist` vs `CRIST, CHARLES JOSEPH` gives 0.58 — a miss. The embedding model, despite scoring only 0.451, at least places the pair in the ambiguous zone where the LLM can confirm the match. Without embeddings, the pair is never surfaced.

### Embed before Clean

If L1 does not decompose names into components, L2 embeds raw strings: `CRIST, CHARLES JOSEPH` as-is. The composite template cannot include `canonical_first` because it does not exist yet. The embedding for the MEDSL record uses `CHARLES` while the OpenElections record uses `Charlie` — the nickname dictionary was never applied. The cosine score drops, more pairs fall below the LLM zone, and matches are lost.

## The General Principle

**Preserve signal as long as possible. Collapse only after all decisions that need the signal have been made.**

Middle initials are signal for disambiguation. Suffixes are signal for generational distinction. Nicknames are signal for matching. Raw strings are signal for provenance. None of these should be discarded until L4, where the entity is already resolved and the canonical name is a presentation choice, not an analytical input.

The pipeline is a funnel of information:

| Layer | Information available | Information consumed |
|-------|---------------------|---------------------|
| L1 | All components: raw, first, middle, last, suffix, canonical_first | None — everything preserved |
| L2 | L1 components + embeddings + quality flags | Components consumed to build composite strings |
| L3 | L2 embeddings + L1 components + LLM context | Embeddings consumed for retrieval; components consumed for LLM reasoning |
| L4 | L3 entity assignments | Entity IDs consumed to select canonical names |

At each layer, information from prior layers is *used* but not *destroyed*. The L1 record persists unchanged alongside the L2, L3, and L4 records. A researcher who disagrees with a canonical name choice can trace back to the original components at L1 and the raw bytes at L0.

## Why This Took a Session to Learn

The old order felt intuitive: clean the data first, then do the hard work. Every data engineering textbook says normalize early. But election entity resolution is not a standard ETL problem. The "dirt" in the data — middle initials, suffixes, nicknames, variant spellings — is not dirt. It is signal. Stripping it is not cleaning. It is destruction.

The key insight: **the order of operations is load-bearing.** Clean → Embed → Match → Canonicalize is the only sequence that preserves signal through the stages that need it and collapses only after all analytical decisions are final.