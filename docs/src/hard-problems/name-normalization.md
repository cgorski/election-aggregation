# Name Normalization

Election data arrives with candidate names in dozens of formats. MEDSL uses `LAST, FIRST MIDDLE` in all caps. NC SBE uses `First Last` in title case. OpenElections uses whatever the county clerk typed. FEC uses `LAST, FIRST MIDDLE SUFFIX`. A single candidate can appear as:

- `CRIST, CHARLES JOSEPH` (MEDSL)
- `Charlie Crist` (OpenElections)
- `Crist, Charlie` (FEC)

These are all the same person. A system that treats them as three different candidates produces garbage output. A system that aggressively normalizes them — stripping middle names, collapsing nicknames, removing suffixes — destroys the signal needed to tell *different* people apart.

The principle: **clean without collapsing.**

## Name decomposition at L1

Every candidate name is decomposed at L1 into six components:

| Component | Purpose | Example |
|-----------|---------|---------|
| `raw` | Original string, unmodified | `CRIST, CHARLES JOSEPH` |
| `first` | Parsed first name | `CHARLES` |
| `middle` | Middle name or initial | `JOSEPH` |
| `last` | Last name | `CRIST` |
| `suffix` | Generational suffix | `null` |
| `canonical_first` | Dictionary-normalized first name | `CHARLES` |

The `canonical_first` field is populated by the [nickname dictionary](./names-dictionary.md). If the raw first name is `Charlie`, `canonical_first` becomes `Charles`. If no mapping exists, `canonical_first` equals `first`.

Both `first` and `canonical_first` are preserved. The raw nickname is useful signal — it tells you what the candidate goes by. The canonical form is what enables matching.

## Real decomposition examples

Five candidates from our prototype, showing how MEDSL and NC SBE formats decompose differently for the same people:

| Source | Raw Name | `first` | `middle` | `last` | `suffix` | `canonical_first` |
|--------|----------|---------|----------|--------|----------|--------------------|
| MEDSL | `DESANTIS, RON` | `RON` | `null` | `DESANTIS` | `null` | `RONALD` |
| OpenElections | `Ron DeSantis` | `Ron` | `null` | `DeSantis` | `null` | `Ronald` |
| MEDSL | `CRIST, CHARLES JOSEPH` | `CHARLES` | `JOSEPH` | `CRIST` | `null` | `CHARLES` |
| OpenElections | `Charlie Crist` | `Charlie` | `null` | `Crist` | `null` | `Charles` |
| MEDSL | `DEMINGS, VAL BUTLER` | `VAL` | `BUTLER` | `DEMINGS` | `null` | `VALDEZ` |
| NC SBE | `Val Demings` | `Val` | `null` | `Demings` | `null` | `Valdez` |
| MEDSL | `WILLIAMS, ROBERT` | `ROBERT` | `null` | `WILLIAMS` | `null` | `ROBERT` |
| NC SBE | `Robert Williams Jr` | `Robert` | `null` | `Williams` | `Jr` | `Robert` |
| MEDSL | `MARSHALL, DAVID S` | `DAVID` | `S` | `MARSHALL` | `null` | `DAVID` |
| MEDSL | `MARSHALL, DAVID A` | `DAVID` | `A` | `MARSHALL` | `null` | `DAVID` |

Key observations from these examples:

1. **Ron DeSantis** — `Ron` maps to `Ronald` via the nickname dictionary. The embedding score between the two source representations is 0.729 — below any reasonable auto-accept threshold, but the LLM matches them using nickname knowledge.

2. **Charlie Crist** — `Charlie` maps to `Charles`. The embedding score is 0.451. Without the dictionary, the cascade would need the LLM to know that Charlie is a nickname for Charles. With the dictionary, the canonical forms already match.

3. **Robert Williams vs Robert Williams Jr** — The suffix `Jr` is the only distinguishing feature. These are different people. The embedding scores them at 0.862 — dangerously close to a false positive. See [Suffixes](./names-suffixes.md).

4. **David S Marshall vs David A Marshall** — Different middle initials. David S. Marshall ran in Maine; David A. Marshall ran in Florida. The middle initial is the *only* signal distinguishing them at the name level. See [Nicknames and Middle Initials](./names-nicknames-and-middles.md).

## What decomposition enables

With names decomposed into components, downstream layers can:

- **Exact-match on structured fields**: `(canonical_first="Timothy", last="Lance", suffix=null)` matches across precincts without fuzzy logic. This handles 70% of entity resolution.
- **Build composite strings for embedding**: `"{canonical_first} {middle} {last} {suffix} | {party} | {office} | {state}"` includes middle initials and suffixes as disambiguation signal.
- **Provide structured context to the LLM**: Instead of asking "are these the same person?", the LLM sees parsed components and can reason about specific differences (nickname vs. different name, Jr vs. no suffix).
- **Block efficiently**: Group by `(state, last_name_initial)` for entity resolution without computing all-pairs similarity.

## What goes wrong without decomposition

If you treat names as opaque strings:

- `CRIST, CHARLES JOSEPH` and `Charlie Crist` have a Jaro-Winkler similarity of 0.58 — a miss.
- `DESANTIS, RON` and `Ron DeSantis` have a cosine embedding similarity of 0.729 — in the ambiguous zone.
- `Robert Williams` and `Robert Williams Jr` look nearly identical to every string metric. Only structured suffix detection prevents a false merge.
- `David S Marshall` and `David A Marshall` differ by one character in a middle initial that opaque matching may ignore entirely.

Decomposition is not optional. It is the foundation that every subsequent layer depends on.

## The three sub-problems

Name normalization breaks into three sub-problems, each with its own chapter:

1. **[Nicknames and Middle Initials](./names-nicknames-and-middles.md)** — How `Charlie` becomes `Charles` and why `David S.` must stay distinct from `David A.`
2. **[Suffixes: Jr/Sr Means Different People](./names-suffixes.md)** — Why generational suffixes are disambiguation signals, not noise to be stripped.
3. **[The Nickname Dictionary](./names-dictionary.md)** — The lookup table that powers `canonical_first`, its current scope, and its limits.