# Office Classification Reference

The pipeline classifies 8,387 unique office name strings into canonical office types using a four-tier system. Each tier handles progressively harder cases. This appendix documents tiers 1 and 2 in full and summarizes tiers 3 and 4.

## Coverage summary

| Tier | Method | Unique offices handled | Cumulative coverage |
|---|---|---|---|
| 1 | Keyword lookup | 3,102 | 37% |
| 2 | Regex patterns | 2,097 | 62% |
| 3 | Embedding similarity | 2,340 | 90% |
| 4 | LLM classification | 848 | 100% |

Tiers 1 and 2 are fully deterministic — same input, same output, no external calls. Tier 3 uses cosine similarity against `text-embedding-3-large` embeddings of known office types. Tier 4 sends unresolved strings to Claude Sonnet with a structured prompt.

## Tier 1: Keyword lookup

A case-insensitive keyword match against the office name string. If any keyword appears in the string, the office is classified immediately. Keywords are checked in order; the first match wins.

| Keyword | `office_level` | `office_category` |
|---|---|---|
| president | federal | executive |
| u.s. senate | federal | legislative |
| u.s. house | federal | legislative |
| congress | federal | legislative |
| governor | state | executive |
| lieutenant governor | state | executive |
| attorney general | state | executive |
| secretary of state | state | executive |
| state treasurer | state | executive |
| state auditor | state | executive |
| state senate | state | legislative |
| state house | state | legislative |
| state representative | state | legislative |
| state assembly | state | legislative |
| supreme court | state | judicial |
| court of appeals | state | judicial |
| appeals court | state | judicial |
| district court | county | judicial |
| superior court | county | judicial |
| county commissioner | county | legislative |
| county council | county | legislative |
| sheriff | county | law_enforcement |
| clerk of court | county | judicial |
| register of deeds | county | administrative |
| coroner | county | administrative |
| constable | county | law_enforcement |
| justice of the peace | county | judicial |
| school board | local | education |
| board of education | local | education |
| city council | local | legislative |
| mayor | local | executive |
| alderman | local | legislative |
| township trustee | local | legislative |
| soil and water | local | special_district |
| fire district | local | special_district |
| water district | local | special_district |

Notes:

- "u.s. senate" is checked before "state senate" to avoid false matches.
- "lieutenant governor" is checked before "governor" for the same reason.
- Keywords are matched as substrings, not whole words. "county commissioner district 3" matches on "county commissioner".

## Tier 2: Regex patterns

When no tier 1 keyword matches, the office string is tested against a series of compiled regular expressions. These handle structural patterns that keyword matching cannot.

| Pattern | `office_level` | `office_category` | Example matches |
|---|---|---|---|
| `(?i)^(us\|united states) (rep\|senator)` | federal | legislative | "US Rep District 4" |
| `(?i)district judge.*district \d+` | county | judicial | "District Judge 21st Judicial District" |
| `(?i)(city\|town\|village) (of\|de) .+ (council\|trustee\|board)` | local | legislative | "Town of Cary Council" |
| `(?i)independent school district.*\d+` | local | education | "Independent School District 279 Board" |
| `(?i)(municipal\|mun\.?) (utility\|water\|sewer) district` | local | special_district | "Municipal Utility District 14" |
| `(?i)community college.*trustee` | local | education | "Community College District Trustee" |
| `(?i)(precinct\|ward) (chair\|committee)` | local | party | "Precinct 12 Committee Chair" |
| `(?i)conservation district (super\|board\|dir)` | local | special_district | "Conservation District Supervisor" |
| `(?i)(drainage\|levee\|flood) (district\|board)` | local | special_district | "Drainage District 7 Board" |
| `(?i)hospital district (board\|dir\|trustee)` | local | special_district | "Hospital District Board Member" |
| `(?i)park (district\|board) (comm\|dir\|trustee)` | local | special_district | "Park District Commissioner" |
| `(?i)sanitary district` | local | special_district | "Sanitary District Trustee" |
| `(?i)mosquito (abatement\|control) district` | local | special_district | "Mosquito Abatement District Trustee" |
| `(?i)(borough\|parish) (council\|president\|assembly)` | county | legislative | "Borough Assembly Member" |
| `(?i)district attorney` | county | law_enforcement | "District Attorney 26th District" |

Regex patterns are tested in order. The first match wins. All patterns use case-insensitive mode.

## Tier 3: Embedding similarity

Office strings that pass through tiers 1 and 2 unclassified are embedded using `text-embedding-3-large` (3072 dimensions) and compared against a reference set of known office type embeddings via FAISS nearest-neighbor search.

- **Threshold:** cosine similarity ≥ 0.85 against the nearest known office type.
- **Reference set:** the canonical office types defined by tiers 1 and 2, plus manually curated additions for jurisdiction-specific titles.
- **Examples resolved at tier 3:**
  - "Moderator" → `local / legislative` (New England town meeting role)
  - "Fence Viewer" → `local / administrative` (historical New England office)
  - "Pound Keeper" → `local / administrative`
  - "Surveyor of Highways" → `local / administrative`
  - "Oyster Commissioner" → `local / special_district` (Maryland)

Tier 3 handles 2,340 unique office strings — mostly jurisdiction-specific titles, historical offices, and compound names that do not match keyword or regex patterns.

## Tier 4: LLM classification

The remaining 848 office strings are sent to Claude Sonnet with a structured prompt that provides the office name, the state, and the county (where available). The LLM returns `office_level`, `office_category`, and a brief rationale.

Every tier 4 decision is recorded in the decision log with:

- `decision_id`
- `input_string` (the original office name)
- `output_level` and `output_category`
- `llm_request_id`
- `rationale` (the LLM's explanation)

Tier 4 classifications can be overridden by adding entries to the tier 1 or tier 2 tables in subsequent pipeline versions. Once an office string is promoted to tier 1 or tier 2, it is classified deterministically on all future runs.

## Office level and category enumerations

**`office_level`** values: `federal`, `state`, `county`, `local`.

**`office_category`** values: `executive`, `legislative`, `judicial`, `law_enforcement`, `administrative`, `education`, `special_district`, `party`.

These enumerations are defined in the [Enumerations Reference](../schema/enumerations.md). Every classified office receives exactly one level and one category.

## Handling ambiguity

Some office strings are genuinely ambiguous:

- "Board of Commissioners" could be county or municipal depending on jurisdiction.
- "Trustee" alone could be township, school board, or special district.
- "Judge" without a court name could be any judicial level.

In these cases, the pipeline uses jurisdiction context (state, county, FIPS code) to disambiguate. If the jurisdiction does not resolve the ambiguity, the string is sent to tier 3 or 4 with the full context attached.