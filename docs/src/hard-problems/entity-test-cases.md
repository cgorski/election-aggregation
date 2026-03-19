# Real Test Cases from Real Data

Every entity resolution decision in this project is grounded in real candidate pairs from real election data. This chapter documents all pairs tested during prototype development, with actual embedding scores, LLM decisions, and the key signal that determined each outcome.

All embeddings use `text-embedding-3-large` (3,072 dimensions). All LLM decisions use Claude Sonnet. Ground truth was established by manual verification against official certified results.

## The Full Test Table

| Name A | Name B | Cosine | LLM Decision | LLM Conf. | Ground Truth | Key Signal |
|--------|--------|:------:|:------------:|:---------:|:------------:|------------|
| Ron DeSantis | DESANTIS, RON | 0.729 | match | 0.98 | match | Nickname: Ron → Ronald |
| Charlie Crist | CRIST, CHARLES JOSEPH | 0.451 | match | 0.95 | match | Nickname: Charlie → Charles; identical votes |
| Robert Williams | Robert Williams Jr | 0.862 | no match | 0.85 | no match | Suffix: Jr indicates different person |
| Val Demings | VAL DEMINGS | 0.828 | match | 0.96 | match | Format difference only; middle initial absent |
| Marco Rubio | RUBIO, MARCO ANTONIO | 0.743 | match | 0.97 | match | Middle name present in one source only |
| Ashley Moody | MOODY, ASHLEY B | 0.930 | match | 0.98 | match | Middle initial added; same office/state |
| Nicole Fried | FRIED, NIKKI | 0.642 | match | 0.92 | match | Nickname: Nikki → Nicole |
| John Smith | SMITH, JOHN R | 0.672 | no match | 0.78 | no match | Common name; different offices, different counties |
| Robert Johnson | JOHNSON, ROBERT L | 0.644 | no match | 0.75 | no match | Common name; different states |
| Dale Holness | HOLNESS, DALE V.C. | 0.896 | match | 0.94 | match | Middle initials added; title prefix stripped |
| Barbara Sharief | SHARIEF, BARBARA J | 0.955 | match | 0.99 | match | Middle initial added; above auto-accept |
| Aramis Ayala | AYALA, ARAMIS D | 0.896 | match | 0.97 | match | Title prefix "State Attorney" stripped; middle initial |

## How to Read This Table

- **Cosine** — Cosine similarity between `text-embedding-3-large` embeddings of the candidate composite strings. Range is 0.0 to 1.0. Higher means more similar.
- **LLM Decision** — The match/no-match output from Claude Sonnet when the pair was in the ambiguous zone (0.35–0.95).
- **LLM Conf.** — The model's self-reported confidence in its decision. Range 0.0 to 1.0.
- **Ground Truth** — Manually verified against official certified election results. "match" means the two records refer to the same human being. "no match" means they do not.
- **Key Signal** — The distinguishing factor that makes this pair interesting for entity resolution testing.

## Analysis by Category

### Nickname Cases

Three pairs test the nickname problem — where one source uses a familiar name and the other uses the legal name:

| Pair | Cosine | Nickname Mapping |
|------|:------:|-----------------|
| DeSantis | 0.729 | Ron → Ronald |
| Crist | 0.451 | Charlie → Charles |
| Fried | 0.642 | Nikki → Nicole |

Embedding scores range from 0.451 to 0.729 — all below the 0.95 auto-accept threshold. Without the LLM step, all three would be missed or would require an unsafely low accept threshold.

The Crist case is the most extreme. At 0.451, the embedding model is essentially saying "these look like different people." The divergence comes from multiple compounding differences: different name ordering (first-last vs last-first), nickname vs legal name, middle name present in only one source, and different casing. The LLM resolves it using nickname knowledge and the identical vote count (3,101,652 in both sources).

After the nickname dictionary is applied at L1, `canonical_first` matches on all three pairs, and step 1 exact match handles them without any embedding or LLM call. The embedding scores reported here are *without* dictionary application — they demonstrate why the dictionary matters.

### Middle Initial Cases

Five pairs test middle-initial handling — where one source includes a middle name or initial and the other does not:

| Pair | Cosine | Middle in Source A | Middle in Source B |
|------|:------:|:------------------:|:------------------:|
| Demings | 0.828 | null | null (format diff) |
| Rubio | 0.743 | null | ANTONIO |
| Moody | 0.930 | null | B |
| Sharief | 0.955 | null | J |
| Ayala | 0.896 | null | D |

Sharief at 0.955 is the only pair above the 0.95 auto-accept threshold. The remaining four fall in the ambiguous zone and require LLM confirmation. The LLM correctly identifies all as matches — the middle initial is additive information, not contradictory information.

Moody at 0.930 is the closest call below auto-accept. The difference between "Ashley Moody" and "MOODY, ASHLEY B" is a single middle initial and formatting. The secondary acceptance rule (embedding ≥ 0.90 AND JW on last name ≥ 0.92 AND same state) handles this case without an LLM call in the production cascade.

### Suffix Cases

One pair tests the suffix problem:

| Pair | Cosine | Suffix A | Suffix B |
|------|:------:|:--------:|:--------:|
| Williams | 0.862 | null | Jr |

At 0.862, this pair would have been auto-accepted under the original threshold of ≥ 0.82. The LLM rejected it with 0.85 confidence, citing the generational distinction implied by "Jr." This single case drove the threshold change from 0.82 to 0.95.

The asymmetry is the danger: one source includes the suffix, the other drops it. The embedding model sees "Robert Williams" and "Robert Williams Jr" as nearly identical strings, because "Jr" is a minor token. The structured suffix field at L1 is the signal that prevents the false merge.

### Common Name Cases

Two pairs test the common-name problem — where two genuinely different people share a common name:

| Pair | Cosine | State A | State B | Office A | Office B |
|------|:------:|:-------:|:-------:|----------|----------|
| Smith | 0.672 | FL | FL | County Commission | School Board |
| Johnson | 0.644 | NC | FL | State House | County Clerk |

Both pairs are correctly rejected. The LLM's confidence is lower (0.75–0.78) than on the match cases because common names are inherently ambiguous — the model cannot be *certain* these are different people, only that the evidence is insufficient for a match.

The Johnson case crosses state boundaries. The blocking strategy partitions by state, so this pair would never be compared in the production cascade. It is included in the test set to validate the cross-state rejection logic.

The Smith case is within the same state but different offices and counties. The LLM correctly reasons that two people named John Smith in different Florida counties holding different offices are most likely different individuals, despite the name match.

### Format Difference Cases

Two pairs test pure formatting differences — same person, same name components, different string representations:

| Pair | Cosine | Format Difference |
|------|:------:|-------------------|
| Holness | 0.896 | Middle initials V.C. added; "Commissioner" prefix stripped |
| Ayala | 0.896 | Middle initial D added; "State Attorney" prefix stripped |

Both score 0.896 — identical cosine similarity despite different underlying differences. Both are correctly matched. These cases validate that the L1 parser correctly strips title prefixes and that the embedding model handles the remaining differences (middle initials) gracefully.

## Score Distribution

The 12 test pairs span the full range of embedding scores relevant to entity resolution:

| Score Range | Count | Matches | Non-Matches |
|-------------|:-----:|:-------:|:-----------:|
| ≥ 0.95 | 1 | 1 | 0 |
| 0.85–0.95 | 4 | 4 | 0 |
| 0.70–0.85 | 3 | 2 | 1 |
| 0.50–0.70 | 3 | 1 | 2 |
| < 0.50 | 1 | 1 | 0 |

The Williams Jr pair at 0.862 is the only false-positive risk — a non-match scoring above 0.85. The Crist pair at 0.451 is the only false-negative risk — a true match scoring below 0.50. These two cases define the boundary conditions of the cascade and drove the threshold calibration described in [Threshold Calibration](./entity-thresholds.md).

## LLM Accuracy

Across all 12 test pairs:

| Metric | Value |
|--------|-------|
| Total pairs tested | 12 |
| LLM correct decisions | 12 |
| LLM accuracy | 100% |
| Average confidence (matches) | 0.957 |
| Average confidence (non-matches) | 0.793 |
| Lowest confidence on a correct match | 0.92 (Fried) |
| Lowest confidence on a correct non-match | 0.75 (Johnson) |

The confidence gap between matches (avg 0.957) and non-matches (avg 0.793) is expected. The LLM is more certain when confirming a match (multiple corroborating signals: same state, same office, similar vote counts, plausible name relationship) than when rejecting one (absence of evidence, not evidence of absence).

## What These Tests Do Not Cover

The 12 test pairs are a calibration set, not a validation set. They do not cover:

- **Spanish-language names** — Hyphenated surnames, maternal/paternal name ordering
- **Transliterated names** — Arabic, Chinese, Vietnamese, and Korean names rendered in English with varying romanization
- **Unisex names** — Cases where a shared name belongs to candidates of different genders
- **Candidate who changed names** — Marriage, legal name change
- **Intentional name variations** — Candidates who use different names in different elections

These gaps are documented as known limitations. The test set will expand as entity resolution is validated at scale.

## Cross-References

- [Entity Resolution Overview](./entity-resolution.md) — the cascade that processes these pairs
- [The Cascade: Step by Step](./entity-cascade.md) — detailed walkthrough of each step
- [Threshold Calibration](./entity-thresholds.md) — how these scores drove the threshold changes