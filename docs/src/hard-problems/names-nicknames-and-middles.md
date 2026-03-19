# Nicknames and Middle Initials

Two distinct problems share a root cause: the candidate's legal name differs from the name on the ballot or in the source file. Nicknames substitute one first name for another. Middle initials appear in some sources and not others. Both must be handled at L1 to preserve signal for L2 and L3.

## Nicknames

A nickname replaces the candidate's legal first name with a familiar variant. The embedding model has no reliable way to recover the connection — it encodes character-level and token-level similarity, not social knowledge about naming conventions.

Real test results from our prototype, using `text-embedding-3-large` (3,072 dimensions):

| Source A | Source B | Nickname → Legal | Cosine | LLM Decision | LLM Confidence |
|----------|----------|-----------------|--------|:------------:|:--------------:|
| Charlie Crist | CRIST, CHARLES JOSEPH | Charlie → Charles | 0.451 | match | 0.95 |
| Nicole Fried | FRIED, NIKKI | Nikki → Nicole | 0.642 | match | 0.92 |
| Ron DeSantis | DESANTIS, RON | Ron → Ronald | 0.729 | match | 0.98 |

The Crist result is the critical case. At 0.451, the embedding score falls below any plausible auto-accept threshold — and below many reject thresholds. Without nickname resolution, this pair would be missed entirely or routed to LLM on every encounter.

The fix operates at L1. The nickname dictionary maps `Charlie` → `Charles`, `Nikki` → `Nicole`, `Ron` → `Ronald`, and ~100 other mappings. When the L1 parser decomposes a name, it checks the first name against the dictionary and populates `canonical_first`:

```json
{
  "raw": "Charlie Crist",
  "first": "Charlie",
  "middle": null,
  "last": "Crist",
  "suffix": null,
  "canonical_first": "Charles"
}
```

Both `first` and `canonical_first` are preserved. The original is kept for display and provenance. The canonical form is used in the L2 composite string for embedding and in the L3 exact-match step. After dictionary application, the L3 exact matcher sees `(canonical_first="Charles", last="Crist", suffix=null)` on both sides — an exact match with no embedding or LLM call required.

### Why the embedding model fails on nicknames

`Charlie` and `Charles` share a prefix, but the embedding model must also reconcile `Crist` vs `CRIST, CHARLES JOSEPH` — different casing, different ordering, and a middle name that appears in one source but not the other. The model embeds the full composite string, not individual tokens. The combined divergence pushes the cosine score to 0.451.

`Ron` and `Ronald` are closer (0.729) because the surface forms are more similar and both sources use last-name-first ordering. But 0.729 is still in the ambiguous zone — it requires an LLM call to confirm.

The nickname dictionary eliminates these LLM calls for known mappings. At scale, this matters: if 5% of candidates use nicknames and each requires an LLM call, that is tens of thousands of unnecessary API round-trips.

## Middle Initials

Middle initials are a different problem. They do not substitute one name for another — they add or remove a disambiguation signal.

The key case: **David S. Marshall** (Maine) and **David A. Marshall** (Florida) are different people. Without middle initials, both reduce to `David Marshall`. With middle initials preserved, L2 generates different embedding vectors.

We measured the effect directly:

| Composite (no middle) | Composite (with middle) | Cosine (no middle) | Cosine (with middle) |
|----------------------|------------------------|:------------------:|:-------------------:|
| David Marshall \| ME | David S Marshall \| ME | — | — |
| David Marshall \| FL | David A Marshall \| FL | 0.7025 | 0.6448 |

The middle initial drops the cosine score by 0.058 — enough to shift the pair further from the accept threshold and closer to correct rejection. The principle: **middle initials are signal, not noise.**

More middle-initial test results from our prototype:

| Source A | Source B | Cosine | LLM Decision | Key Signal |
|----------|----------|:------:|:------------:|-----------|
| Ashley Moody | Ashley B. Moody | 0.930 | match | Same person, middle added |
| Val Demings | VAL DEMINGS | 0.828 | match | Same person, format difference |
| Dale Holness | DALE V.C. HOLNESS | 0.896 | match | Same person, middle initials added |

Ashley Moody at 0.930 is the same person — the `B.` appears in one source but not the other. The high embedding score plus same-state context is sufficient for auto-accept above the 0.95 threshold (or just below it, in which case JW on the last name at 1.0 pushes it through).

## How Both Feed Into L2

The L2 composite string for a candidate includes both `canonical_first` and `middle`:

```text
{canonical_first} {middle} {last} {suffix} | {party} | {office} | {state} | {county}
```

For Charlie Crist, this becomes:

```text
Charles  Crist  | DEM | Governor | FL | statewide
```

For CRIST, CHARLES JOSEPH, this becomes:

```text
Charles Joseph Crist  | DEM | Governor | FL | statewide
```

The canonical first names now match. The remaining divergence — `Joseph` as a middle name in one source — is small enough that the embedding score rises well above the ambiguous zone. The nickname dictionary at L1 did the heavy lifting; L2 and L3 finish the job.

## The Combined Rule

1. At L1, apply the nickname dictionary to populate `canonical_first`.
2. At L1, preserve `middle` exactly as parsed — do not strip it, do not normalize it.
3. At L2, include both `canonical_first` and `middle` in the composite string.
4. At L3 exact match, match on `(canonical_first, last, suffix)` — middle is not required for exact match but is used for disambiguation when multiple candidates share the same canonical first and last name.
5. At L3 LLM confirmation, provide both the raw and canonical names so the model can reason about nickname relationships and middle-initial differences.

The principle behind both: **clean without collapsing.** Normalize what you can (nicknames to canonical forms), preserve what you must (middle initials as disambiguation signal), and let downstream layers use the full context.