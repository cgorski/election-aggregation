# Office Classification

MEDSL 2022 contains **8,387 unique office names** across all 50 states and DC. These are not 8,387 distinct offices — they are 8,387 different strings that humans typed to describe elected positions. "Board of Education", "BOARD OF ED.", "BOE", "School Board", and "Board of Education Members" all refer to the same type of office. "DALLAS COUNTY JUDGE" means a chief executive in Texas and a judicial officer everywhere else.

Classifying these strings into a consistent taxonomy is required for every downstream operation: blocking for entity resolution, computing competitiveness by office type, comparing the same office across states, and answering "what offices exist in my county?"

## The taxonomy

Every office is classified into two fields:

| Field | Values | Example |
|-------|--------|---------|
| `office_level` | `federal`, `state`, `county`, `municipal`, `school_district`, `special_district`, `judicial`, `tribal` | `school_district` |
| `office_branch` | `executive`, `legislative`, `judicial`, `law_enforcement`, `fiscal`, `education`, `infrastructure`, `regulatory`, `other` | `education` |

The pair `(office_level, office_branch)` defines the classification. "Board of Education" → `(school_district, education)`. "County Sheriff" → `(county, law_enforcement)`. "City Council" → `(municipal, legislative)`.

## The scale of the problem

Of the 8,387 unique office names in MEDSL 2022:

| Characteristic | Count | Percentage |
|---------------|------:|----------:|
| Appear in only 1 state | 6,241 | 74.4% |
| Appear in only 1 county | 4,995 | 59.6% |
| Appear in 10+ states | 312 | 3.7% |
| Contain a proper noun (county/city name) | 3,108 | 37.1% |

Most office names are effectively unique strings. "DALLAS COUNTY JUDGE", "Collier Mosquito Control District", "Santa Rosa Island Authority" — these appear once in the entire national dataset. No keyword list can enumerate them all. The classifier must generalize.

## Four-tier approach

The classifier runs four tiers in sequence. Each tier handles what the previous tier could not. A record classified at tier 1 is never re-examined by tier 2.

| Tier | Method | Unique names handled | Cumulative % | Cost |
|------|--------|--------------------:|-----------:|------|
| 1 | Keyword lookup | ~3,775 | ~45.0% | $0 |
| 2 | Regex patterns | ~1,426 | ~62.0% | $0 |
| 3 | Embedding nearest-neighbor | ~378 | ~66.5% | ~$0.01/1K |
| 4 | LLM classification | ~42 | ~67.0% | ~$0.002/call |
| — | Unclassified (`other`) | ~2,766 | 100% | — |

The remaining ~33% classified as `other` are primarily hyper-local offices (township-specific roles, water district sub-boards, tribal offices) that require either expanded reference data or manual review. The `other` rate drops as the keyword and regex lists expand.

> **Note:** Percentages are based on unique office name strings. By *record count*, the coverage is much higher — the 312 names that appear in 10+ states account for millions of records. Keyword tier 1 alone handles ~85% of records by volume.

## Tier 1: Keyword lookup

A table of ~170 keywords mapped to `(office_level, office_branch)` pairs. If any keyword appears in the office name string, the classification is assigned.

| Keyword | office_level | office_branch | Example match |
|---------|-------------|---------------|---------------|
| `sheriff` | county | law_enforcement | "WARREN COUNTY SHERIFF" |
| `board of education` | school_district | education | "COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02" |
| `city council` | municipal | legislative | "CITY COUNCIL WARD 3" |
| `coroner` | county | fiscal | "COUNTY CORONER" |
| `constable` | county | law_enforcement | "CONSTABLE PRECINCT 4" |

Keywords are matched case-insensitively. When multiple keywords match, the most specific wins ("county board of education" matches `board of education` → school_district, not `county` → county). The keyword table is maintained in the appendix.

Keyword lookup handles approximately 45% of unique office name strings and ~85% of total records. The most common offices — sheriff, school board, city council, county commission — all have unambiguous keywords.

## Tier 2: Regex patterns

Approximately 40 regex patterns handle structured variations that keywords miss. Patterns capture positional and combinatorial relationships:

| Pattern | office_level | office_branch | Example match |
|---------|-------------|---------------|---------------|
| `county\s+commission` | county | legislative | "CLARK COUNTY COMMISSION DIST 2" |
| `district\s+court\s+judge` | judicial | judicial | "15TH DISTRICT COURT JUDGE" |
| `register\s+of\s+(deeds\|wills)` | county | fiscal | "REGISTER OF DEEDS" |
| `soil.*water.*conservation` | special_district | infrastructure | "SOIL AND WATER CONSERVATION DISTRICT SUPERVISOR" |
| `(mayor\|alcalde)` | municipal | executive | "MAYOR - CITY OF SPRINGFIELD" |

Regex patterns add approximately 17% of unique names beyond what keywords catch. Combined with tier 1, the two deterministic tiers handle ~62% of unique names and ~92% of records by volume.

## Tier 3: Embedding nearest-neighbor

For names that survive tiers 1 and 2, L2 generates an embedding using `text-embedding-3-large` and finds the nearest neighbor in a reference set of ~200 pre-classified office names.

Real example from our prototype:

- **Input:** "Collier Mosquito Control District"
- **Nearest neighbor:** "Mosquito Control District" (reference set)
- **Cosine similarity:** 0.787
- **Classification:** `(special_district, infrastructure)`

The tier 3 accept threshold is cosine ≥ 0.60. Below that, the match is too uncertain and the record passes to tier 4. In our prototype, tier 3 classified ~4.5% of remaining unique names with a manual-review accuracy of 94%.

The 200-name reference set was curated from the most common office names across all states, covering every `(office_level, office_branch)` pair with at least 3 reference examples. Expanding this set to 500+ names is a planned improvement.

## Tier 4: LLM classification

Remaining unclassified names go to Claude Sonnet with the full context: office name, state, county, and the taxonomy definition.

Real examples from our prototype:

| Office name | State | LLM classification | Confidence |
|-------------|-------|-------------------|:----------:|
| Santa Rosa Island Authority | FL | special_district / infrastructure | 0.90 |
| Mosquito Control Board Member | FL | special_district / infrastructure | 0.95 |
| Judge of Compensation Claims | FL | judicial / judicial | 0.88 |
| Public Administrator | MO | county / fiscal | 0.82 |
| Recorder of Deeds | MO | county / fiscal | 0.95 |
| Drainage Commissioner | IL | special_district / infrastructure | 0.85 |
| Fence Viewer | VT | municipal / regulatory | 0.70 |
| Pound Keeper | NH | municipal / regulatory | 0.65 |
| Hog Reeve | NH | municipal / regulatory | 0.60 |

In our prototype, the LLM classified 9 hard cases with **100% accuracy** against manual review. The lower-confidence cases (Fence Viewer at 0.70, Hog Reeve at 0.60) are genuine obscure New England town offices that even the LLM finds unusual — but it classified them correctly.

## The state-context problem

"DALLAS COUNTY JUDGE" illustrates why state context matters. In Texas, the county judge is the presiding officer of the commissioners court — an executive role, not a judicial one. In every other state, a county judge sits on the bench.

The keyword classifier alone cannot resolve this. The word "judge" appears, suggesting `judicial`. But the Texas county judge is `(county, executive)`.

The fix is a state-specific override table in tier 1. Before general keyword matching, a small set of (state, keyword) → classification entries handles known exceptions:

| State | Office pattern | Correct classification |
|-------|---------------|----------------------|
| TX | county judge | county / executive |
| LA | parish president | county / executive |
| LA | police jury | county / legislative |
| AK | borough assembly | county / legislative |

This table is currently small (~15 entries). As more state-specific offices are identified, it grows. The pattern generalizes: when the same word means different things in different states, the state-specific override takes priority.

## Accuracy by tier

| Tier | Method | Accuracy (manual review) | False positive rate |
|------|--------|:-----------------------:|:------------------:|
| 1 | Keyword | 99.2% | < 0.5% |
| 2 | Regex | 97.8% | ~1.0% |
| 3 | Embedding NN | 94.0% | ~3.5% |
| 4 | LLM | 100% (N=9) | 0% (N=9) |

Tier 1 and 2 errors are almost entirely from the state-context problem (a keyword matching the wrong sense of the word). Tier 3 errors come from embedding matches that are semantically close but functionally wrong — "Tax Collector" matching to "Tax Assessor" when they are separate offices in some states.

## Cross-references

- [The Four-Tier Classifier](./office-four-tiers.md) — step-by-step walkthrough with a single office name through all four tiers
- [Appendix: Office Classification Reference](../appendix/office-classification.md) — full keyword table and regex pattern list