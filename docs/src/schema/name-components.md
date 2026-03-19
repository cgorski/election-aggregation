# Candidate Name Components

Election data sources represent candidate names as a single string. The formats are incompatible across sources — and sometimes within the same source across years. The pipeline decomposes every name into structured components at L1 and preserves all components through every subsequent layer.

## Why decomposition instead of a single string

A single `name` field cannot support entity resolution. Consider matching these records:

| Source | Raw name string |
|--------|----------------|
| MEDSL | `SHANNON W BRAY` |
| NC SBE | `Shannon W. Bray` |
| FEC | `BRAY, SHANNON W` |

String equality fails on all three pairs. Lowercasing and stripping punctuation gets MEDSL and NC SBE closer, but FEC's last-first ordering still breaks. Decomposing into `{first: Shannon, middle: W, last: Bray}` makes all three identical after normalization.

The harder case is nicknames:

| Source | Raw name string | What a human sees |
|--------|----------------|-------------------|
| MEDSL | `MICHAEL "STEVE" HUBER` | First name Michael, goes by Steve |
| NC SBE | `Michael (Steve) Huber` | Same person |
| OpenElections | `Steve Huber` | Same person, nickname only |

Without decomposition, matching `Steve Huber` to `MICHAEL "STEVE" HUBER` requires the system to know that Steve is a nickname present in one variant but used as the primary name in another. The `nickname` and `canonical_first` fields make this explicit.

## Component fields

Every candidate name in the pipeline is represented as a struct with seven fields:

| Field | Type | Description | Populated at |
|-------|------|-------------|-------------|
| `raw` | `String` | Original name string exactly as it appeared in the source. Never modified. | L1 |
| `first` | `Option<String>` | Parsed first name | L1 |
| `middle` | `Option<String>` | Parsed middle name or initial | L1 |
| `last` | `Option<String>` | Parsed last name | L1 |
| `suffix` | `Option<String>` | Generational suffix: Jr, Sr, II, III, IV | L1 |
| `nickname` | `Option<String>` | Detected nickname, extracted from quotes or parentheses | L1 |
| `canonical_first` | `Option<String>` | Nickname-resolved first name. If `first` has a known nickname mapping, this holds the canonical form. | L1 |

All fields are available at every layer (L1 through L4). Later layers may refine values but never discard earlier ones.

## Parsing rules by source

### MEDSL

Names are ALL CAPS, no periods after initials, nicknames in double quotes, suffixes without commas.

| Raw | first | middle | last | suffix | nickname | canonical_first |
|-----|-------|--------|------|--------|----------|-----------------|
| `SHANNON W BRAY` | `Shannon` | `W` | `Bray` | — | — | `Shannon` |
| `MICHAEL "STEVE" HUBER` | `Michael` | — | `Huber` | — | `Steve` | `Michael` |
| `ROBERT VAN FLETCHER JR` | `Robert` | `Van` | `Fletcher` | `Jr` | — | `Robert` |
| `LM "MICKEY" SIMMONS` | `L` | `M` | `Simmons` | — | `Mickey` | `L` |
| `VICTORIA P PORTER` | `Victoria` | `P` | `Porter` | — | — | `Victoria` |
| `WRITEIN` | — | — | — | — | — | — |

`WRITEIN` is a sentinel value, not a person name. It is flagged at L1 and excluded from name decomposition.

### NC SBE

Names are Title Case, periods after initials, nicknames in parentheses, commas before suffixes.

| Raw | first | middle | last | suffix | nickname | canonical_first |
|-----|-------|--------|------|--------|----------|-----------------|
| `Shannon W. Bray` | `Shannon` | `W` | `Bray` | — | — | `Shannon` |
| `Michael (Steve) Huber` | `Michael` | — | `Huber` | — | `Steve` | `Michael` |
| `Robert Van Fletcher, Jr.` | `Robert` | `Van` | `Fletcher` | `Jr` | — | `Robert` |
| `Patricia (Pat) Cotham` | `Patricia` | — | `Cotham` | — | `Pat` | `Patricia` |
| `William Irvin. Enzor III` | `William` | `Irvin` | `Enzor` | `III` | — | `William` |

The period after "Irvin." in the last example is a data entry artifact. The parser strips trailing periods from middle names.

### FEC

Names are LAST, FIRST MIDDLE format, all caps.

| Raw | first | middle | last | suffix | nickname | canonical_first |
|-----|-------|--------|------|--------|----------|-----------------|
| `BRAY, SHANNON W` | `Shannon` | `W` | `Bray` | — | — | `Shannon` |
| `BIDEN, JOSEPH R JR` | `Joseph` | `R` | `Biden` | `Jr` | — | `Joseph` |

## The `canonical_first` field

`canonical_first` resolves known nicknames to their formal equivalents using the [nickname dictionary](../appendix/nickname-dictionary.md). This enables matching when one source uses a nickname and another uses the legal name.

| first | nickname | canonical_first | Reasoning |
|-------|----------|-----------------|-----------|
| `Michael` | `Steve` | `Michael` | First name is already formal |
| `Charlie` | — | `Charles` | Charlie is a known nickname for Charles |
| `Bob` | — | `Robert` | Bob is a known nickname for Robert |
| `Patricia` | `Pat` | `Patricia` | First name is already formal |
| `Bill` | — | `William` | Bill is a known nickname for William |
| `Jim` | — | `James` | Jim is a known nickname for James |

When `first` is already a formal name, `canonical_first` equals `first`. When `first` is itself a nickname (as when OpenElections reports `Charlie Crist` without the legal name `Charles`), `canonical_first` resolves to the formal form.

The nickname dictionary contains approximately 1,200 mappings. It is deterministic — no ML, no API calls. Ambiguous cases (e.g., "Alex" could map to "Alexander" or "Alexandra") are resolved by leaving `canonical_first` equal to `first` and deferring to embedding-based matching at L2.

## How L2 uses name components

L2 constructs a composite string for embedding from the decomposed components:

```
{canonical_first} {middle} {last} {suffix}
```

This means `Michael "Steve" Huber` and `Steve Huber` both embed with their decomposed components rather than raw strings. The embedding model sees structured, normalized text rather than source-specific formatting.

The `raw` field is never used for embedding. It is preserved for provenance and debugging only.

## Special cases

**Write-in candidates.** MEDSL aggregates write-ins into `WRITEIN`. NC SBE reports named write-ins (e.g., `Ronnie Strickland (Write-In)`) separately from `Write-In (Miscellaneous)`. Named write-ins are decomposed normally. The `WRITEIN` sentinel produces a record with all name fields set to `None`.

**Ballot measure choices.** The values `For`, `Against`, `Yes`, `No` are not person names. They are handled by the `BallotMeasure` contest kind and bypass name decomposition entirely. See [Contest Kinds](./contest-kinds.md).

**Hyphenated last names.** Treated as a single `last` value: `Smith-Jones` → `last: Smith-Jones`. No attempt is made to split on hyphens.

**Multiple middle names.** Concatenated into the `middle` field: `Joseph Robinette Biden` → `middle: Robinette`. If two middle names are present (rare), they are space-separated in the `middle` field.

**No first name.** Some sources report only a last name (e.g., `WRITEIN` or truncated records). `first` is `None`. `canonical_first` is also `None`.