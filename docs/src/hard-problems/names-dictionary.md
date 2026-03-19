# The Nickname Dictionary

The nickname dictionary is a static lookup table applied at L1 during name decomposition. It maps common short names and nicknames to their formal equivalents, populating the `canonical_first` field while preserving the original `first` field unchanged.

## Scope

The prototype dictionary contains approximately 100 mappings covering the most frequent English-language nicknames encountered in US election data:

| Raw first | canonical_first | Frequency in MEDSL 2022 |
|-----------|----------------|------------------------|
| Bill | William | 847 |
| Bob | Robert | 612 |
| Jim | James | 589 |
| Mike | Michael | 534 |
| Charlie | Charles | 201 |
| Ron | Ronald | 187 |
| Nikki | Nicole | 42 |
| Ted | Edward | 31 |
| Dick | Richard | 28 |
| Peggy | Margaret | 19 |

The target for production is 500+ mappings, expanding to cover Spanish-language nicknames (Pepe→José, Pancho→Francisco), regional variants, and less common English forms. The full reference list is maintained in [Appendix: Full Nickname Dictionary](../appendix/nickname-dictionary.md).

## Both forms are preserved

When the dictionary maps `Charlie` → `Charles`, the L1 record stores both:

```json
{
  "first": "Charlie",
  "canonical_first": "Charles"
}
```

The original `first` is never overwritten. The composite string sent to L2 embedding uses `canonical_first`, which is why the embedding for "Charles Crist" and "CRIST, CHARLES JOSEPH" can be compared at all — even though the raw cosine similarity between "Charlie Crist" and "CRIST, CHARLES JOSEPH" is only 0.451.

## The Ted problem

Some nicknames are ambiguous. "Ted" can map to Edward (Ted Kennedy) or Theodore (Ted Cruz). "Bill" is unambiguous — it always maps to William. "Ted" is not.

The current dictionary maps Ted → Edward, which is the more common historical usage in US politics. This is wrong for Theodore-named candidates. The correct resolution requires context that L1 does not have: party, state, office, or a reference database of known candidates.

The planned fix is a two-pass approach: L1 applies the majority mapping (Ted → Edward), and L3 entity resolution can override it when the LLM has enough context to determine the correct expansion. The `canonical_first` field is treated as a best guess at L1, not a final answer.

Other ambiguous nicknames with the same property: Pat (Patricia or Patrick), Chris (Christopher or Christine), Alex (Alexander or Alexandra), Sam (Samuel or Samantha). For these, L1 does not apply a mapping — `canonical_first` is left equal to `first` — and disambiguation is deferred to L3.