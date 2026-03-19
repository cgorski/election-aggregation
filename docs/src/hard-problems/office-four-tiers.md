# The Four-Tier Classifier

Office classification proceeds through four tiers in strict order. Each tier handles a progressively harder subset of the 8,387 unique office names found in MEDSL 2022. A name classified at tier 1 never reaches tier 2. A name classified at tier 2 never reaches tier 3. The tiers are ordered by cost: deterministic and free first, embedding-based second, LLM last.

## Tier 1: Keyword Match

A lookup table of 170 keyword entries maps office name substrings to `(office_level, office_branch)` pairs. Matching is case-insensitive and checks for substring containment.

**Example:**

Raw office name: `WARREN COUNTY BOARD OF EDUCATION`

The keyword table contains:

| Keyword | office_level | office_branch |
|---------|-------------|--------------|
| board of education | school_district | education |

`"board of education"` appears as a substring → classified as `school_district/education`.

**Coverage:** ~3,775 of 8,387 unique names (~45.0%). These are the offices with unambiguous keywords: sheriff, coroner, board of education, city council, state senate, district court, county clerk, school board, mayor, constable, treasurer.

**Limitations:** Keyword matching is context-free. `DALLAS COUNTY JUDGE` contains `judge`, which maps to `county/judicial`. In Texas, the County Judge is the chief executive — `county/executive` is correct. Tier 1 gets this wrong. The planned fix is a state-context override table applied before keyword matching.

## Tier 2: Regex Patterns

Approximately 40 regular expressions handle office names with structural patterns that keywords alone cannot capture.

**Example:**

Raw office name: `CLERK OF THE CIRCUIT COURT, 11TH JUDICIAL CIRCUIT`

Regex pattern: `clerk\s+of\s+(the\s+)?(circuit|district|superior)\s+court`

Match → classified as `county/judicial`.

Other regex examples:

| Pattern | Matches | Classification |
|---------|---------|---------------|
| `county\s+commission` | County Commissioner, County Commission District 3 | county/legislative |
| `(city\|town\|village)\s+council` | City Council Ward 2, Town Council At Large | municipal/legislative |
| `district\s+\d+\s+judge` | District 14 Judge, District 3 Judge | county/judicial |
| `soil\s+and\s+water` | Soil and Water Conservation District Supervisor | special_district/conservation |

**Coverage:** ~1,426 additional unique names (~17.0%), bringing the cumulative total to ~62.0%.

**Limitations:** Regex patterns are brittle against novel phrasings. `CONSERVATION DISTRICT BOARD MEMBER` does not match the soil-and-water pattern. Regex also cannot handle the 4,995 office names that appear in exactly one county — writing a pattern for each is infeasible.

## Tier 3: Embedding Nearest Neighbor

The remaining ~3,186 unclassified office names are embedded using `text-embedding-3-large` and compared against a reference set of ~200 pre-classified office names. The nearest neighbor's classification is assigned if cosine similarity exceeds 0.60.

**Example:**

Raw office name: `Collier Mosquito Control District`

Nearest reference: `Mosquito Control District` → `special_district/infrastructure`

Cosine similarity: **0.787**

0.787 > 0.60 → classified as `special_district/infrastructure` with confidence 0.787.

Other tier 3 results:

| Unclassified Name | Nearest Reference | Cosine | Classification |
|-------------------|------------------|:------:|---------------|
| Collier Mosquito Control District | Mosquito Control District | 0.787 | special_district/infrastructure |
| Eastern Carrituck Fire & Rescue | Fire Protection District | 0.724 | special_district/infrastructure |
| Lowndes County Bd of Ed | Board of Education | 0.831 | school_district/education |
| Hospital Authority Board | Hospital District | 0.692 | special_district/health |

**Coverage:** ~378 additional unique names (~4.5%), bringing the cumulative total to ~66.5%.

**What falls through:** Office names with no close reference analog, names below the 0.60 threshold, and names whose nearest neighbor is misleading (e.g., `Community Development District` matching `Community College District` at 0.71 — wrong classification). These proceed to tier 4.

## Tier 4: LLM Classification

The final tier sends unclassified office names to Claude Sonnet with a structured prompt that includes the office name, state, and the full taxonomy of `(office_level, office_branch)` pairs.

**Example:**

Raw office name: `Santa Rosa Island Authority`

State: Florida

The LLM prompt provides the taxonomy and asks: "Classify this office into the most appropriate (office_level, office_branch) pair. Explain your reasoning."

LLM response:

> **Classification:** special_district/infrastructure (confidence: 0.90)
>
> "The Santa Rosa Island Authority is a special-purpose governmental entity in Escambia County, Florida, responsible for managing development and infrastructure on Santa Rosa Island (Pensacola Beach). It is not a general-purpose county or municipal government. 'Special district' at the 'infrastructure' branch is the best fit."

**Coverage:** ~42 additional unique names (~0.5%) in our prototype evaluation, classified with 100% accuracy against manual review (9 of 9 hard cases correct).

Other tier 4 examples:

| Office Name | State | LLM Classification | Confidence |
|-------------|-------|-------------------|:----------:|
| Santa Rosa Island Authority | FL | special_district/infrastructure | 0.90 |
| Cuyahoga County Executive | OH | county/executive | 0.95 |
| Drainage Commissioner | IL | special_district/infrastructure | 0.85 |
| Register of Mesne Conveyances | SC | county/judicial | 0.88 |

The South Carolina example is illustrative: "Register of Mesne Conveyances" is an office that exists in exactly one state. No keyword, regex, or embedding reference can classify it without external knowledge. The LLM knows that mesne conveyances is a legal term related to property transfers and that the Register is a judicial officer.

## Tier Summary

| Tier | Method | Unique Names | Cumulative % | Cost per Name | Deterministic |
|------|--------|:------------:|:------------:|:-------------:|:------------:|
| 1 | Keyword (170 entries) | ~3,775 | 45.0% | $0 | Yes |
| 2 | Regex (~40 patterns) | ~1,426 | 62.0% | $0 | Yes |
| 3 | Embedding NN (200 refs) | ~378 | 66.5% | ~$0.0001 | Yes* |
| 4 | LLM | ~42 | 67.0% | ~$0.001 | No |
| — | Unclassified / `other` | ~2,766 | 100% | — | — |

\* Deterministic given the same embedding model version.

The remaining ~33% classified as `other` are office names that did not pass through our full pipeline in the prototype. At production scale, tiers 1–4 are projected to handle ~99.5% of names, with ~0.5% remaining as `other` pending human review.

## Why Four Tiers Instead of Just the LLM

Three reasons:

1. **Speed.** Keyword and regex classify 62% of names in microseconds. Embedding NN classifies 4.5% more in milliseconds. Sending all 8,387 names to the LLM would take minutes and achieve the same result for the easy cases.

2. **Reproducibility.** Tiers 1–3 produce identical output on every run. Tier 4 may produce slightly different reasoning (though classifications are stable in practice). Minimizing non-deterministic surface area makes the pipeline easier to audit.

3. **Debuggability.** When a classification is wrong, the `classifier_method` field tells you which tier produced it. A wrong keyword mapping is a one-line table fix. A wrong regex is a pattern edit. A wrong embedding match means the reference set needs expansion. A wrong LLM classification means the prompt needs refinement. Each failure mode has a distinct fix.

## Cross-Reference

- [Office Classification Overview](./office-classification.md) — the 8,387-name problem and tier coverage statistics
- [Appendix: Office Classification Reference](../appendix/office-classification.md) — full keyword and regex lists
- [L1: Cleaned](../architecture/l1-cleaned.md) — where tiers 1–2 run
- [L2: Embedded](../architecture/l2-embedded.md) — where tier 3 runs
- [When the LLM Gets Called](../architecture/llm-when.md) — tier 4 invocation policy