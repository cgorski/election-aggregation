# Verify a Specific Result

**Question:** Can I verify that "Timothy Lance got 303 votes in precinct P17"? Can I trace that number back to the original source file?

Yes. The hash chain links every L4 canonical record back through L3, L2, and L1 to the raw bytes of the L0 source file. This recipe walks the chain step by step using `jq`.

## The Claim

A researcher sees this record in the L4 flat export:

```text
Timothy Lance — 303 votes — Precinct P17 — Columbus County Schools Board of Education District 02 — NC — 2022-11-08
```

They want to verify it. Here is how.

## Step 1: Find the L4 Record

Start at the L4 flat export and locate the record:

```sh
jq -c 'select(
  .candidate_canonical == "Timothy Lance"
  and .county == "COLUMBUS"
  and .votes_total == 303
)' l4_canonical/exports/flat_export.jsonl
```

Output:

```text
{"election_date":"2022-11-08","state":"NC","county":"COLUMBUS","contest_name":"COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02","candidate_canonical":"Timothy Lance","candidate_entity_id":"person:nc:columbus:lance-timothy-13","votes_total":303,"source":"nc_sbe","l3_hash":"28183d41d50204d5","l0_hash":"edfedf2760cfd54f"}
```

Note the two hash values:
- `l3_hash`: `28183d41d50204d5` — links to the L3 matched record
- `l0_hash`: `edfedf2760cfd54f` — shortcut to the L0 source file

## Step 2: Follow l3_hash to L3

Look up the L3 matched record by its hash:

```sh
jq -c 'select(.l3.l3_hash == "28183d41d50204d5")' \
  l3_matched/NC/2022/matched.jsonl
```

Key fields in the output:

```text
{
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

This tells you:
- The entity resolution cascade assigned Timothy Lance to entity `person:nc:columbus:lance-timothy-13`.
- The contest was assigned to `contest:nc:columbus:school-board-d02`.
- The L2 parent hash is `854fa6367960bb05`.

## Step 3: Follow l2_parent_hash to L2

Look up the L2 embedded record:

```sh
jq -c 'select(.l2.l2_hash == "854fa6367960bb05")' \
  l2_embedded/NC/2022/enriched.jsonl
```

Key fields:

```text
{
  "l2": {
    "l2_hash": "854fa6367960bb05",
    "l1_parent_hash": "8ea7ecc257ff8e05",
    "embedding_model": "text-embedding-3-large",
    "embedding_dimensions": 3072,
    "candidate_composite": "Timothy Lance | | BOARD OF EDUCATION DISTRICT 02 | NC | Columbus",
    "quality_flags": []
  }
}
```

This tells you:
- The embedding model was `text-embedding-3-large` with 3,072 dimensions.
- The composite string used for embedding was `Timothy Lance | | BOARD OF EDUCATION DISTRICT 02 | NC | Columbus`.
- No quality flags were raised.
- The L1 parent hash is `8ea7ecc257ff8e05`.

## Step 4: Follow l1_parent_hash to L1

Look up the L1 cleaned record:

```sh
jq -c 'select(.provenance.l1_hash == "8ea7ecc257ff8e05")' \
  l1_cleaned/nc_sbe/NC/2022/cleaned.jsonl
```

Key fields:

```text
{
  "jurisdiction": {
    "state": "NC", "state_fips": "37",
    "county": "COLUMBUS", "county_fips": "37047",
    "precinct": "P17"
  },
  "results": [{
    "candidate_name": {
      "raw": "Timothy Lance", "first": "Timothy",
      "middle": null, "last": "Lance",
      "suffix": null, "canonical_first": "Timothy"
    },
    "votes_total": 303,
    "vote_counts_by_type": {
      "election_day": 136, "early": 159,
      "absentee_mail": 7, "provisional": 1
    }
  }],
  "provenance": {
    "l1_hash": "8ea7ecc257ff8e05",
    "l0_parent_hash": "edfedf2760cfd54f",
    "parser_version": "nc_sbe_v2.1",
    "schema_version": "3.0.0"
  }
}
```

This tells you:
- The 303 votes break down to 136 election day + 159 early + 7 absentee + 1 provisional.
- The name was parsed as first="Timothy", last="Lance", no middle, no suffix.
- The parser version was `nc_sbe_v2.1`.
- The L0 parent hash is `edfedf2760cfd54f`.

## Step 5: Follow l0_parent_hash to L0

Look up the L0 manifest:

```sh
cat l0_raw/nc_sbe/results_pct_20221108.txt.manifest.json
```

Output:

```text
{
  "l0_hash": "edfedf2760cfd54f",
  "source_url": "https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2022_11_08/results_pct_20221108.zip",
  "retrieval_date": "2026-03-18T14:30:00Z",
  "file_size_bytes": 18023456,
  "format_detected": "tsv"
}
```

## Step 6: Verify L0 Against the Source

Recompute the SHA-256 of the raw file and compare:

```sh
# macOS
shasum -a 256 l0_raw/nc_sbe/results_pct_20221108.txt

# Linux
sha256sum l0_raw/nc_sbe/results_pct_20221108.txt
```

If the output starts with `edfedf2760cfd54f...`, the raw file is intact — it matches the bytes the pipeline processed.

To verify against the authoritative source independently, download the file yourself:

```sh
curl -O https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2022_11_08/results_pct_20221108.zip
unzip results_pct_20221108.zip
shasum -a 256 results_pct_20221108.txt
```

If your hash matches the manifest's `l0_hash`, you and the pipeline processed identical bytes. The vote count of 303 for Timothy Lance in precinct P17 traces directly to those bytes.

## The Full Chain

```text
L4  flat_export.jsonl
    candidate_canonical = "Timothy Lance", votes_total = 303
    l3_hash = 28183d41d50204d5
      │
L3  matched.jsonl
    entity_id = person:nc:columbus:lance-timothy-13
    l2_parent_hash = 854fa6367960bb05
      │
L2  enriched.jsonl
    embedding_model = text-embedding-3-large
    candidate_composite = "Timothy Lance | | BOARD OF EDUCATION DISTRICT 02 | NC | Columbus"
    l1_parent_hash = 8ea7ecc257ff8e05
      │
L1  cleaned.jsonl
    votes_total = 303 (136 + 159 + 7 + 1)
    parser_version = nc_sbe_v2.1
    l0_parent_hash = edfedf2760cfd54f
      │
L0  results_pct_20221108.txt
    l0_hash = edfedf2760cfd54f
    source_url = https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2022_11_08/results_pct_20221108.zip
```

Every link is independently verifiable. Recompute any hash from the record content plus the parent hash. If it matches the stored value, the record has not been tampered with.

## Prototype Validation

In our 200-record prototype, we verified the full hash chain for every record:

| Metric | Result |
|--------|--------|
| Records verified | 200 / 200 |
| Broken chains | 0 |
| Layers traversed | 5 per record |
| Total hash verifications | 1,000 |

Zero broken links. Every vote count traces back to the raw NC SBE file bytes.

## When to Use This

- **Fact-checking.** A journalist writing "Timothy Lance received 303 votes" can cite the hash chain as evidence.
- **Auditing.** A researcher who finds an unexpected result can walk the chain to determine whether the issue is in the source data (L0), the parser (L1), the entity resolution (L3), or the aggregation (L4).
- **Dispute resolution.** If two researchers disagree on a number, both can verify the chain. If both chains are intact and both start from the same L0 hash, the number is correct. If the L0 hashes differ, one of them has a different version of the source file — check `retrieval_date` in the manifest.