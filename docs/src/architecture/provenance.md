# Provenance and the Hash Chain

Every record at every layer carries a cryptographic hash of its own content and a pointer to its parent layer's hash. This chain links any L4 canonical export record back through L3 matching, L2 embedding, and L1 cleaning to the exact bytes of the original source file at L0. If any record at any layer is modified — a vote count changed, a name altered, a match decision overridden — the chain breaks at precisely that point.

## The Hash Structure

Each layer computes its hash as:

```text
l{N}_hash = SHA-256( record_content + "parent:" + l{N-1}_hash )
```

The `record_content` is the deterministic serialization of all fields at that layer (excluding the hash itself). The `parent:` prefix is a literal string separator. The parent hash anchors the current record to its predecessor.

```text
L4 canonical record
  l4_hash ← SHA-256(L4 content + "parent:" + l3_hash)
    │
    └── L3 matched record
          l3_hash ← SHA-256(L3 content + "parent:" + l2_hash)
            │
            └── L2 embedded record
                  l2_hash ← SHA-256(L2 content + "parent:" + l1_hash)
                    │
                    └── L1 cleaned record
                          l1_hash ← SHA-256(L1 content + "parent:" + l0_hash)
                            │
                            └── L0 raw file
                                  l0_hash ← SHA-256(raw file bytes)
```

## A Real Example: Timothy Lance Through All Five Layers

Timothy Lance ran for Columbus County Schools Board of Education District 02 in the 2022 NC general election. Here is one of his precinct-level records traced through every layer.

### L0: Raw

The NC SBE results file `results_pct_20221108.txt` is stored byte-identical at `l0_raw/nc_sbe/results_pct_20221108.txt`.

```json
{
  "l0_hash": "edfedf2760cfd54f",
  "source_url": "https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2022_11_08/results_pct_20221108.zip",
  "retrieval_date": "2026-03-18T14:30:00Z",
  "file_size_bytes": 18023456,
  "format_detected": "tsv"
}
```

The `l0_hash` is the SHA-256 of the raw file bytes (truncated here for display). Re-downloading the file and re-hashing produces the same value. If NC SBE updates the file after our retrieval, the hash changes and a new L0 entry is created.

### L1: Cleaned

The NC SBE parser extracts Timothy Lance's precinct P17 row and produces a structured record:

```json
{
  "jurisdiction": {
    "state": "NC", "county": "COLUMBUS", "precinct": "P17"
  },
  "contest": {
    "raw_name": "COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02",
    "office_level": "school_district"
  },
  "results": [{
    "candidate_name": {
      "raw": "Timothy Lance", "first": "Timothy",
      "middle": null, "last": "Lance",
      "suffix": null, "canonical_first": "Timothy"
    },
    "votes_total": 303
  }],
  "provenance": {
    "l1_hash": "8ea7ecc257ff8e05",
    "l0_parent_hash": "edfedf2760cfd54f",
    "parser_version": "nc_sbe_v2.1",
    "schema_version": "3.0.0"
  }
}
```

The `l1_hash` is computed from the L1 record content plus `"parent:edfedf2760cfd54f"`. The `l0_parent_hash` links back to the raw file.

### L2: Embedded

L2 generates a composite string and embedding for the candidate:

```json
{
  "l2": {
    "l2_hash": "854fa6367960bb05",
    "l1_parent_hash": "8ea7ecc257ff8e05",
    "embedding_model": "text-embedding-3-large",
    "embedding_dimensions": 3072,
    "candidate_embedding_id": 4271,
    "candidate_composite": "Timothy Lance | | BOARD OF EDUCATION DISTRICT 02 | NC | Columbus",
    "quality_flags": []
  }
}
```

The `l2_hash` is computed from the L2 fields plus `"parent:8ea7ecc257ff8e05"`. The `l1_parent_hash` links back to L1.

### L3: Matched

Entity resolution assigns a `candidate_entity_id`. Timothy Lance appeared identically across all precincts, so step 1 (exact match) resolved him:

```json
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

The `l3_hash` is computed from the L3 fields plus `"parent:854fa6367960bb05"`.

### L4: Canonical

L4 produces the researcher-facing export record:

```json
{
  "election_date": "2022-11-08",
  "state": "NC",
  "county": "COLUMBUS",
  "contest_name": "COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02",
  "candidate_canonical": "Timothy Lance",
  "candidate_entity_id": "person:nc:columbus:lance-timothy-13",
  "votes_total": 303,
  "source": "nc_sbe",
  "l4_hash": "f19a3e8bc7210d42",
  "l3_hash": "28183d41d50204d5",
  "l0_hash": "edfedf2760cfd54f"
}
```

The `l4_hash` is computed from the L4 fields plus `"parent:28183d41d50204d5"`. The record also carries `l0_hash` as a shortcut for end-to-end verification.

## Verification Algorithm

To verify a single L4 record:

1. Read the L4 record. Recompute `SHA-256(L4 content + "parent:" + l3_hash)`. Compare to stored `l4_hash`. If mismatch → **chain broken at L4**.
2. Look up the L3 record by `l3_hash`. Recompute `SHA-256(L3 content + "parent:" + l2_hash)`. Compare to stored `l3_hash`. If mismatch → **chain broken at L3**.
3. Look up the L2 record by `l2_hash`. Recompute `SHA-256(L2 content + "parent:" + l1_hash)`. Compare to stored `l2_hash`. If mismatch → **chain broken at L2**.
4. Look up the L1 record by `l1_hash`. Recompute `SHA-256(L1 content + "parent:" + l0_hash)`. Compare to stored `l1_hash`. If mismatch → **chain broken at L1**.
5. Read the L0 raw file. Recompute `SHA-256(file bytes)`. Compare to stored `l0_hash`. If mismatch → **chain broken at L0** (source file was modified or corrupted).

If all five checks pass, the record is verified from canonical output back to original source bytes.

## Prototype Results

In our 200-record prototype run:

| Metric | Result |
|--------|--------|
| Records verified | 200 / 200 |
| Broken chains | 0 |
| Layers traversed per record | 5 (L4 → L3 → L2 → L1 → L0) |
| Total hash verifications | 1,000 (200 records × 5 layers) |

Every hash chain verified end-to-end with zero broken links.

## What Breaks the Chain

The hash chain detects any modification at any layer. Specific scenarios:

**Modifying a vote count at L1.** If someone changes Timothy Lance's votes from 303 to 304, the L1 content changes, the recomputed `l1_hash` no longer matches the stored value, and the L2 record's `l1_parent_hash` no longer points to a valid L1 record.

**Changing a parser without a version bump.** If the NC SBE parser is updated but `parser_version` is not incremented, the L1 content for existing records may change (different parsing logic applied to the same raw bytes). The `l1_hash` changes, breaking the chain from L2 upward. The `parser_version` field exists precisely to prevent silent parser changes.

**Overriding an L3 match decision.** If a human reviewer changes an entity assignment at L3, the `l3_hash` changes. L4 must be re-run from the amended L3 output. The original L3 decision is preserved in the decision log — it is never deleted, only superseded.

**Re-downloading a source file after the publisher updated it.** NC SBE occasionally corrects results files after initial publication. If the corrected file has different bytes, the `l0_hash` changes. The entire pipeline from L1 upward must be re-run for affected records. The original L0 entry and its manifest are retained as a versioned snapshot.

## Why Not a Merkle Tree

A Merkle tree would allow verifying subsets of records without recomputing the full chain. We use a simpler linear chain because:

1. **Records are independent.** Each precinct-level record has its own chain. Verifying one record does not require knowledge of any other record. A Merkle tree adds complexity without benefit when records are not aggregated into blocks.

2. **Full verification is cheap.** SHA-256 of a 2 KB record takes microseconds. Verifying all 200 records takes less than a second. At 200 million records, full verification takes minutes — well within acceptable bounds for a batch pipeline.

3. **Simplicity aids trust.** A journalist verifying a specific result needs to understand "follow the hash backward through five files." A Merkle tree requires understanding tree structure, sibling hashes, and root computation. The simpler model is more auditable by non-engineers.

## The Chain as Documentation

The hash chain is not just an integrity mechanism — it is a documentation trail. Every L4 record answers the question: "Where did this number come from?" Follow `l3_hash` to see which entity resolution decision assigned this candidate ID. Follow `l2_parent_hash` to see the embedding and composite string. Follow `l1_parent_hash` to see the parsed record. Follow `l0_parent_hash` to see the raw source file.

This is provenance in the literal sense: the origin and chain of custody of every data point, cryptographically verifiable.