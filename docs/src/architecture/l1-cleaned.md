# L1: Cleaned — Deterministic Parsing and Enrichment

L1 transforms raw source files into structured JSONL records with a unified schema. It is purely deterministic: no machine learning, no API calls, no randomness. Given the same L0 files and the same parser version, L1 output is identical on every run, on every machine, forever.

This is deliberate. L1 is the foundation for every subsequent layer. If the foundation is non-deterministic, nothing above it can be reproduced.

## One Parser Per Source, One Schema Out

Each source has a dedicated parser that understands its native format:

| Source | Format | Delimiter | Encoding | Parser |
|--------|--------|-----------|----------|--------|
| NC SBE | TSV (`.txt` extension) | `\t` | UTF-8 | `nc_sbe_v2.1` |
| MEDSL | CSV | `,` | UTF-8 | `medsl_v1.3` |
| OpenElections | CSV (varies by state) | `,` | UTF-8/Latin-1 | `openelections_v1.0` |
| Clarity/Scytl | XML | — | UTF-8 | `clarity_v0.5` |

Every parser produces the same output schema. A downstream consumer of L1 JSONL does not need to know whether a record originated from NC SBE or MEDSL — the fields, types, and semantics are identical.

## The 10 Operations

L1 applies 10 operations in fixed order. The order matters — later operations depend on earlier ones.

### 1. Filter Non-Contests

Before any parsing, detect rows that are not candidate results. Pattern-match on the candidate name field:

| Pattern | Classification | Action |
|---------|---------------|--------|
| `registered voters` | TurnoutMetadata | Extract to `turnout.registered_voters` |
| `ballots cast` | TurnoutMetadata | Extract to `turnout.ballots_cast` |
| `over votes` | TurnoutMetadata | Extract to `turnout.over_votes` |
| `under votes` | TurnoutMetadata | Extract to `turnout.under_votes` |
| `^blank$` | TurnoutMetadata | Maine's undervote label |
| `total votes` | Aggregation artifact | Discard (redundant with candidate sums) |
| `for` / `against` / `yes` / `no` | BallotMeasure (if contest name matches) | Route to `MeasureChoice` |

This runs first because non-contest rows must not enter name decomposition, office classification, or entity resolution. The principle is **extract before filter** — the registered voter count is valuable turnout data and is captured before the row is excluded from candidate analysis. See [Non-Candidate Records](../hard-problems/non-candidates.md).

### 2. Parse Source Format

Source-specific column mapping. The NC SBE parser reads tab-separated fields: `County`, `Election Date`, `Contest Name`, `Choice`, `Choice Party`, `Total Votes`, `Election Day`, `One Stop`, `Absentee by Mail`, `Provisional`. The MEDSL parser reads CSV columns: `state`, `county_name`, `office`, `candidate`, `party_simplified`, `votes`, `mode`. Each parser maps its native columns to the unified schema fields.

Encoding normalization happens here. OpenElections files from some states use Latin-1 encoding; the parser detects and converts to UTF-8. MEDSL 2022 has trailing commas in some state files; the parser strips them.

### 3. Decompose Candidate Names

Split every candidate name into structured components. This is the most critical L1 operation — it determines what signal survives to L2 and L3.

The decomposition handles four source formats:

| Format | Example | Parsing strategy |
|--------|---------|-----------------|
| `LAST, FIRST MIDDLE` | `CRIST, CHARLES JOSEPH` | Split on first comma; remainder is first + middle |
| `First Last` | `Charlie Crist` | Last token is last name (with multi-word last name detection) |
| `First Middle Last Suffix` | `Robert Van Fletcher, Jr.` | Suffix detected and extracted; remaining tokens parsed |
| `LAST, FIRST M.` | `BRAY, SHANNON W.` | Period stripped from middle initial |

The output for every format is the same six fields:

```json
{
  "raw": "Robert Van Fletcher, Jr.",
  "first": "Robert",
  "middle": "Van",
  "last": "Fletcher",
  "suffix": "Jr.",
  "canonical_first": "Robert"
}
```

Every component is preserved. Middle initials are kept (they distinguish David S. Marshall from David A. Marshall). Suffixes are kept (they distinguish Robert Williams from Robert Williams Jr.). The `raw` field is never modified. See [Name Normalization](../hard-problems/name-normalization.md).

### 4. Apply Nickname Dictionary

Look up `first` in the nickname dictionary (~100 mappings in prototype, targeting 500+). If a mapping exists, populate `canonical_first` with the formal equivalent. If not, `canonical_first` equals `first`.

| `first` | `canonical_first` | Mapping |
|---------|-------------------|---------|
| Charlie | Charles | Charlie → Charles |
| Ron | Ronald | Ron → Ronald |
| Nikki | Nicole | Nikki → Nicole |
| Timothy | Timothy | No mapping (already formal) |

Both fields are preserved. The composite string sent to L2 uses `canonical_first`; the original `first` is retained for display and provenance. See [The Nickname Dictionary](../hard-problems/names-dictionary.md).

### 5. Classify Contest Kind

Route each record to one of three contest kinds based on signals from steps 1 and 2:

| Kind | Criteria | Example |
|------|----------|---------|
| `candidate_race` | Default — a person running for office | Timothy Lance for Board of Education |
| `ballot_measure` | Candidate name is For/Against/Yes/No AND contest name matches measure keywords | "Against" in "BOND REFERENDUM" |
| `turnout_metadata` | Candidate name matches turnout patterns | "Registered Voters" |

Records classified as `ballot_measure` get a `MeasureChoice` result instead of `CandidateResult`. Records classified as `turnout_metadata` are extracted and attached to sibling precinct records.

### 6. Classify Office (Tiers 1–2)

Apply the deterministic tiers of the [office classifier](../hard-problems/office-classification.md):

**Tier 1: Keyword lookup** (~170 entries). Case-insensitive substring match. `"board of education"` in the contest name → `school_district/education`. Handles ~45% of unique office names, ~85% of records by volume.

**Tier 2: Regex patterns** (~40 patterns). `county\s+commission` → `county/legislative`. Adds ~17% of unique names.

Records that do not match either tier are classified as `other` with `classifier_confidence: 0.0`. They proceed to L2 for tier 3 (embedding nearest-neighbor) and tier 4 (LLM classification).

The `classifier_method` field records which tier produced the classification: `"keyword"`, `"regex"`, or `"unclassified"`.

### 7. Enrich Geography

Look up FIPS codes from bundled Census Bureau reference data:

- **State FIPS**: 2-digit code from state abbreviation. `NC` → `37`.
- **County FIPS**: 5-digit code from (state, county name). `(NC, COLUMBUS)` → `37047`.
- **Place FIPS**: Where available, municipal codes from Census place files.
- **OCD-ID**: Open Civic Data identifier. `ocd-division/country:us/state:nc/county:columbus`.

The reference data covers 3,143 counties and 31,980 places. FIPS enrichment achieves 100% county coverage for records with valid state and county name fields. Municipal FIPS coverage is lower (~85%) because municipality names are less standardized.

### 8. Compute Vote Shares

For each candidate in a contest within a precinct:

```text
vote_share = votes_total / sum(votes_total for all candidates in same contest+precinct)
```

Vote share is a convenience field — it can always be recomputed from the raw vote counts. It is included because downstream queries (margins, competitiveness rankings) use it constantly.

### 9. Backfill Turnout

If step 1 extracted turnout metadata rows for a precinct, attach the values to all sibling contest records in the same precinct:

```json
{
  "turnout": {
    "registered_voters": 4217,
    "ballots_cast": 2891,
    "turnout_rate": 0.6855
  }
}
```

Turnout data is available in NC SBE and some OpenElections files. It is absent from MEDSL and Clarity. When absent, the `turnout` field is null — not zero, not omitted, but explicitly null to distinguish "no data" from "zero registered voters."

### 10. Compute L1 Hash

The final operation seals the record into the hash chain:

```text
l1_hash = SHA-256( serialize(record_without_hash) + "parent:" + l0_hash )
```

The `l0_hash` comes from the L0 manifest of the source file. The `l1_hash` becomes the anchor for L2. See [Provenance and the Hash Chain](./provenance.md).

## A Real L1 Record

Timothy Lance, precinct P17, Columbus County Schools Board of Education District 02, 2022 NC general election:

```json
{
  "election": {"date": "2022-11-08", "type": "general"},
  "jurisdiction": {
    "state": "NC", "state_fips": "37",
    "county": "COLUMBUS", "county_fips": "37047",
    "precinct": "P17", "level": "precinct"
  },
  "contest": {
    "kind": "candidate_race",
    "raw_name": "COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02",
    "office_level": "school_district",
    "classifier_method": "regex",
    "classifier_confidence": 0.85,
    "vote_for": 1
  },
  "results": [
    {
      "candidate_name": {
        "raw": "Timothy Lance", "first": "Timothy",
        "middle": null, "last": "Lance", "suffix": null,
        "canonical_first": "Timothy"
      },
      "votes_total": 303,
      "vote_share": 0.523,
      "vote_counts_by_type": {
        "election_day": 136, "early": 159,
        "absentee_mail": 7, "provisional": 1
      }
    }
  ],
  "turnout": {
    "registered_voters": 4217,
    "ballots_cast": 2891,
    "turnout_rate": 0.6855
  },
  "source": {
    "source_type": "nc_sbe",
    "source_file": "results_pct_20221108.txt",
    "confidence": "high"
  },
  "provenance": {
    "l1_hash": "8ea7ecc257ff8e05",
    "l0_parent_hash": "edfedf2760cfd54f",
    "parser_version": "nc_sbe_v2.1",
    "schema_version": "3.0.0"
  }
}
```

Every field traces to a specific operation: `county_fips` from step 7, `canonical_first` from step 4, `office_level` from step 6, `turnout` from step 9, `l1_hash` from step 10.

## What L1 Does Not Do

- **No embeddings.** Embedding generation requires an API call to OpenAI. L1 runs offline with zero external dependencies.
- **No entity resolution.** L1 does not determine whether two records refer to the same person. That is L3's job.
- **No canonical name selection.** L1 preserves all name components. Choosing the "best" name is L4's job, after entity resolution.
- **No tier 3/4 office classification.** Embedding-based and LLM-based classification require API calls. L1 applies only the deterministic tiers (keyword and regex). Records that need tiers 3–4 are marked `"classifier_method": "unclassified"` and classified at L2.

This boundary is the determinism boundary. Everything L1 does can be verified by re-running the parser on the same L0 files. No API key, no network connection, no randomness.