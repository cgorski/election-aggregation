# The Five-Layer Pipeline

The pipeline processes election data through five immutable layers. Each layer depends on all prior layers. Every record carries a hash chain back to the original source bytes. The storage format at every layer is JSONL (one JSON record per line).

```text
L0  RAW         Byte-identical source files with acquisition manifests.
 │
 │  deterministic parse — no ML, no API calls
 ▼
L1  CLEANED     Structured records. Names decomposed into components.
 │              FIPS enrichment. Office classification (keyword + regex).
 │
 │  deterministic given embedding model version
 ▼
L2  EMBEDDED    Vector embeddings for candidates, contests, geography.
 │              Tier 3 office classification. Quality flags.
 │
 │  non-deterministic — LLM decisions stored in audit log
 ▼
L3  MATCHED     Entity resolution. candidate_entity_id assigned.
 │              contest_entity_id assigned. Cross-source dedup.
 │
 │  deterministic given L3 entity assignments
 ▼
L4  CANONICAL   Authoritative names. Temporal chains. Alias tables.
                Verification algorithms. Researcher-facing exports.
```

## Layer properties

| Layer | Deterministic | Needs API key | Output format | Re-runnable from |
|-------|:------------:|:-------------:|---------------|-----------------|
| L0 | Yes | No | Original files + `.manifest.json` | External sources |
| L1 | Yes | No | JSONL | L0 |
| L2 | Yes, given model version | Yes (OpenAI) | JSONL + `.npy` sidecars | L1 |
| L3 | No (LLM) | Yes (Anthropic) | JSONL + decision log (JSONL) | L2 |
| L4 | Yes, given L3 | No | JSONL + JSON registries + CSV export | L3 |

The determinism boundary falls between L2 and L3. Everything from L0 through L2 produces identical output from identical input, given the same code version and embedding model. L3 introduces LLM calls whose outputs may vary between runs, but every decision is stored in a JSONL audit log that enables deterministic replay.

## What each layer produces

### L0: Raw

The input to the entire pipeline. L0 is a byte-identical copy of each source file, accompanied by a JSON manifest recording how it was acquired.

```text
l0_raw/
├── nc_sbe/
│   ├── results_pct_20221108.txt           # Original file, untouched
│   └── results_pct_20221108.txt.manifest.json
├── medsl/
│   └── 2022-nc-local-precinct-general/
│       ├── NC-cleaned-final3.csv
│       └── NC-cleaned-final3.csv.manifest.json
└── ...
```

The manifest records:

```json
{
  "l0_hash": "edfedf2760cfd54f...",
  "source_url": "https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2022_11_08/results_pct_20221108.zip",
  "retrieval_date": "2026-03-18T14:30:00Z",
  "file_size_bytes": 18023456,
  "format_detected": "tsv"
}
```

L0 files are never modified. If a source is re-downloaded and the content differs, a new versioned L0 entry is created.

### L1: Cleaned

L1 parses each source's native format into a unified JSONL schema. The parser is source-specific (one parser per source), but the output schema is the same regardless of source.

L1 performs 10 operations in fixed order:

1. **Filter non-contests** — Detect "Registered Voters", "Ballots Cast", "Over Votes", "Under Votes". Route to turnout metadata. Detect "For"/"Against"/"Yes"/"No" ballot measure choices.
2. **Parse source format** — Source-specific: CSV for MEDSL, TSV for NC SBE, XML for Clarity.
3. **Decompose candidate names** — Split into first, middle, last, suffix, nickname. Preserve every component. `Robert Van Fletcher, Jr.` becomes `{first: "Robert", middle: "Van", last: "Fletcher", suffix: "Jr.", raw: "Robert Van Fletcher, Jr."}`.
4. **Apply nickname dictionary** — Map `Charlie` → `Charles`, `Bill` → `William`, etc. Store as `canonical_first`. Preserve original `first`.
5. **Classify contest kind** — CandidateRace, BallotMeasure, or TurnoutMetadata.
6. **Classify office (tiers 1–2)** — Keyword lookup (~170 entries), then regex patterns (~40 patterns). No ML, no embeddings. Records that don't match remain `other`.
7. **Enrich geography** — FIPS lookup from bundled reference data (3,143 counties, 31,980 places). Generate OCD-IDs.
8. **Compute vote shares** — `votes_total / sum(all candidates in contest)`.
9. **Backfill turnout** — If turnout metadata rows were found, attach registered voter counts to sibling contest records in the same precinct.
10. **Compute L1 hash** — `SHA-256(record content + "parent:" + L0 hash)`.

A single L1 record for a Columbus County, NC school board race:

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
      "vote_counts_by_type": {
        "election_day": 136, "early": 159,
        "absentee_mail": 7, "provisional": 1
      }
    }
  ],
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

L1 does not use any machine learning, API calls, or non-deterministic processes. Given the same L0 files and the same parser version, L1 output is identical on every run.

### L2: Embedded

L2 generates vector embeddings for text fields that need fuzzy matching. The embedding model is `text-embedding-3-large` (3,072 dimensions) from OpenAI. L2 also applies tier 3 office classification (embedding nearest-neighbor against a reference set of ~200 known office names) and raises quality flags on suspicious records.

L2 produces three types of output:

**Enriched JSONL** — L1 records augmented with classification upgrades and quality flags:

```json
{
  "...all L1 fields...",
  "l2": {
    "l2_hash": "854fa6367960bb05",
    "l1_parent_hash": "8ea7ecc257ff8e05",
    "embedding_model": "text-embedding-3-large",
    "embedding_dimensions": 3072,
    "candidate_embedding_id": 4271,
    "contest_embedding_id": 183,
    "candidate_composite": "Timothy Lance | | BOARD OF EDUCATION DISTRICT 02 | NC | Columbus",
    "contest_composite": "COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02 | school_district | NC 2022",
    "quality_flags": []
  }
}
```

**Embedding sidecars** — Binary `.npy` files (float32 arrays) containing the actual vectors. One file per embedding type per partition:

```text
l2_embedded/
├── nc/2022/
│   ├── enriched.jsonl
│   ├── candidate_embeddings.npy    # float32[N, 3072]
│   ├── contest_embeddings.npy      # float32[M, 3072]
│   └── geography_embeddings.npy    # float32[K, 3072]
```

**ID mapping** — A JSON file mapping L1 record hashes to embedding row indices.

The composite strings fed to the embedding model follow fixed templates:

| Purpose | Template |
|---------|----------|
| Candidate | `{canonical_first} {middle} {last} {suffix} \| {party} \| {office} \| {state} \| {county}` |
| Contest | `{raw_contest_name} \| {office_level} \| {state} {year}` |
| Geography | `{municipality}, {county} County, {state}` |

Middle initials and suffixes are included in the candidate composite. This is deliberate — "David S Marshall" and "David A Marshall" produce different vectors, which helps distinguish different people with the same first and last name. We measured this: including the middle initial reduced cosine similarity between the two David Marshalls from 0.7025 to 0.6448.

L2 is deterministic given the same embedding model version. If OpenAI changes the weights behind `text-embedding-3-large`, the vectors change. The `embedding_model` and model version are stored in every L2 record to detect this.

### L3: Matched

L3 resolves entities — it determines which records across sources and elections refer to the same candidate and the same contest. This is the first non-deterministic layer because it uses LLM calls for ambiguous cases.

L3 runs an entity resolution cascade for each candidate record:

| Step | Method | Handles | Cost |
|------|--------|---------|------|
| 1 | Exact match on `(canonical_first, last, suffix)` | Same name across precincts | $0 |
| 2 | Jaro-Winkler similarity ≥ 0.92 | Minor spelling variations | $0 |
| 2.5 | Name similarity gate: JW on last name < 0.50 → skip | Obvious non-matches | $0 |
| 3 | Embedding retrieval: cosine ≥ 0.95 → auto-accept | Format differences | $0 |
| 4 | LLM confirmation: cosine 0.35–0.95 | Nicknames, suffixes, ambiguous names | ~$0.0002/call |
| 5 | Tiebreaker: stronger model when step 4 is uncertain | Low-confidence cases | ~$0.002/call |

In our prototype run of 200 records:
- Step 1 resolved 597 candidates (70.0%)
- Step 2 resolved 1 (0.1%)
- Step 3 resolved 50 (5.9%)
- Step 4 was invoked 30 times (3.5%), all resulting in no-match
- 206 unique candidate entities were created

The 30 LLM calls in our prototype were all spent on pairs within the same (state, office_level) block that had moderate embedding similarity (0.55–0.73) but completely different names — "Aaron Bridges" vs "Daniel Blanton" type comparisons. All 30 were correctly rejected. This finding led to the addition of step 2.5 (the name similarity gate): if the Jaro-Winkler score on last names alone is below 0.50, skip the pair entirely without computing embedding similarity.

Every L3 decision is stored in a JSONL audit log:

```json
{
  "decision_id": "a3f8c1d2-...",
  "decision_type": "candidate_match",
  "timestamp": "2026-03-19T10:30:00Z",
  "inputs": {
    "name_a": "Charlie Crist",
    "name_b": "CRIST, CHARLES JOSEPH",
    "embedding_score": 0.451,
    "state_a": "FL", "state_b": "FL",
    "contest_a": "Governor", "contest_b": "Governor",
    "votes_a": 3101652, "votes_b": 3101652
  },
  "method": {
    "type": "llm",
    "model": "claude-sonnet-4-20250514",
    "prompt_template_version": "entity_match_v2.0"
  },
  "output": {
    "decision": "match",
    "confidence": 0.95,
    "reasoning": "Charlie is a common nickname for Charles. Same state, same office, identical vote counts."
  }
}
```

A researcher who wants to reproduce L3 can either replay the cached decisions from the log (deterministic) or re-run the LLM calls (which may produce slightly different responses). The log preserves everything needed for either approach.

L3 adds entity assignments to each record:

```json
{
  "...all L2 fields...",
  "l3": {
    "l3_hash": "28183d41d50204d5",
    "l2_parent_hash": "854fa6367960bb05",
    "candidate_entity_ids": [
      {"result_index": 0, "entity_id": "person:nc:columbus:lance-timothy-13"}
    ],
    "contest_entity_id": "contest:nc:columbus:school-board-d02"
  }
}
```

### L4: Canonical

L4 assigns authoritative representations. For each entity (candidate or contest), it selects a canonical name, builds temporal chains across elections, constructs alias tables, and runs verification algorithms.

**Canonical name selection** follows a fixed algorithm:

1. Collect all name variants from all L3 records in the entity cluster.
2. Prefer the most complete variant (one with a middle initial over one without; one with a suffix over one without).
3. Among equally complete variants, prefer the one from the most authoritative source (certified state data > academic data > community data).
4. Among equally authoritative variants, prefer the most recent.

**Temporal chains** aggregate records by (entity_id, election_date, contest_entity_id). One entry per election, not per precinct. A candidate who appeared in 47 precincts in one election gets one temporal chain entry with the summed vote total.

**Verification algorithms** run at L4 to check pipeline integrity:

1. **Hash chain integrity** — Walk L4→L3→L2→L1→L0 for every record. Verify no link is broken.
2. **Entity consistency** — Flag entities spanning multiple states (unusual for local officials). Flag party switches.
3. **Temporal plausibility** — Flag implausible career spans or office progressions.
4. **Cross-source reconciliation** — Where two sources cover the same contest, compare vote totals.
5. **Completeness audit** — Report coverage by state, county, year. Report FIPS and entity ID fill rates.
6. **LLM entity audit** — For multi-member entities, ask a language model whether the cluster is plausible. In our prototype, this caught 43 suspicious entities (precinct-level records inflating temporal chains) and 4 likely errors ("For" and "Against" ballot measure choices classified as person entities).

L4 exports two types of output:

**Entity registries** (JSON) — One record per unique person or contest:

```json
{
  "entity_id": "person:nc:columbus:lance-timothy-13",
  "canonical_name": "Timothy Lance",
  "aliases": ["Timothy Lance", "TIMOTHY LANCE"],
  "elections": [
    {"date": "2022-11-08", "contest": "Columbus County Schools Board of Education District 02", "votes": 1531}
  ],
  "states": ["NC"],
  "first_appearance": "2022-11-08",
  "election_count": 1
}
```

**Flat exports** (JSONL and CSV) — One record per candidate per contest per precinct, with canonical names and entity IDs attached:

```json
{
  "election_date": "2022-11-08",
  "state": "NC",
  "county": "COLUMBUS",
  "contest_name": "COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02",
  "candidate_raw": "Timothy Lance",
  "candidate_canonical": "Timothy Lance",
  "candidate_entity_id": "person:nc:columbus:lance-timothy-13",
  "votes_total": 303,
  "source": "nc_sbe",
  "l3_hash": "28183d41d50204d5",
  "l0_hash": "edfedf2760cfd54f"
}
```

## Why five layers and not two

A simpler system would have two layers: raw and processed. The five-layer design exists because the processing steps have different properties that should not be conflated:

**Splitting L1 from L2** means you can upgrade the embedding model without re-parsing all sources. If a better model than `text-embedding-3-large` becomes available, re-run L2 from L1. L1 remains untouched.

**Splitting L2 from L3** means cheap, deterministic embedding generation is separate from expensive, non-deterministic LLM calls. L2 can run for 200 million records in hours on CPU (plus API calls for vector generation). L3's LLM calls can be batched separately, retried on failure, and audited independently.

**Splitting L3 from L4** means individual entity resolution decisions are separate from the aggregate operations (canonical name selection, temporal chains, verification) that consume them. If a human reviewer overrides an L3 match decision, L4 can be re-run without re-doing all of L3.

Each layer boundary is a point where you can stop, inspect, export, and restart. A researcher who disagrees with the entity resolution can take L2 output and apply their own matching logic. A developer who wants to test a new office classifier can re-run L1 without re-downloading L0.

## Storage layout

```text
local-data/processed/
├── l0_raw/
│   └── {source}/
│       ├── {filename}
│       └── {filename}.manifest.json
├── l1_cleaned/
│   └── {source}/{state}/{year}/
│       ├── cleaned.jsonl
│       └── cleaning_report.json
├── l2_embedded/
│   └── {state}/{year}/
│       ├── enriched.jsonl
│       ├── candidate_embeddings.npy
│       ├── contest_embeddings.npy
│       └── id_mapping.json
├── l3_matched/
│   └── {state}/{year}/
│       ├── matched.jsonl
│       └── decisions/
│           └── candidate_matches.jsonl
└── l4_canonical/
    ├── candidate_registry.json
    ├── contest_registry.json
    ├── verification_report.json
    └── exports/
        ├── flat_export.jsonl
        └── flat_export.csv
```

All JSONL files are streamable — they can be processed line by line without loading the entire file into memory. At 200 million records with approximately 2 KB per record, the full L1 corpus would be approximately 400 GB. Streaming is not optional at that scale.

## The hash chain

Every record at every layer carries a hash of its own content and a reference to its parent layer's hash:

```text
L4 record
  l4_hash ← SHA-256(L4 content + "parent:" + l3_hash)
    └── l3_hash ← SHA-256(L3 content + "parent:" + l2_hash)
          └── l2_hash ← SHA-256(L2 content + "parent:" + l1_hash)
                └── l1_hash ← SHA-256(L1 content + "parent:" + l0_hash)
                      └── l0_hash ← SHA-256(raw file bytes)
```

To verify any L4 record: recompute the L4 hash from its content, check that it matches the stored `l4_hash`, then follow the `l3_parent_hash` to the L3 record and repeat. Continue through L2 and L1 to L0. At L0, re-hash the raw file bytes and compare to the stored `l0_hash`.

In our prototype run of 200 records, all 200 hash chains verified from L4 back to L0 with zero broken links.