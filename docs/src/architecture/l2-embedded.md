# L2: Embedded — Vector Generation and Classification

L2 transforms L1's structured text fields into vector embeddings suitable for fuzzy matching, applies tier 3 office classification, and raises quality flags on suspicious records. It is the bridge between deterministic parsing (L1) and probabilistic entity resolution (L3).

## Embedding Model

The embedding model is OpenAI's `text-embedding-3-large`, producing 3,072-dimensional float32 vectors. Every L2 record stores the model identifier and dimensionality:

```json
{
  "embedding_model": "text-embedding-3-large",
  "embedding_dimensions": 3072
}
```

This metadata is not optional. Thresholds calibrated for `text-embedding-3-large` (auto-accept ≥ 0.95, ambiguous 0.35–0.95, auto-reject < 0.35) are not portable to other models. If the model changes, the thresholds must be recalibrated against the [test cases](../hard-problems/entity-test-cases.md). Storing the model in every record ensures that stale thresholds are never applied to vectors from a different model.

## Composite String Templates

Raw name components are not embedded directly. They are assembled into composite strings that include contextual fields — office, state, county, party — so that the resulting vectors encode identity-relevant context alongside the name.

Three composite types are generated per record:

| Type | Template | Example |
|------|----------|---------|
| Candidate | `{canonical_first} {middle} {last} {suffix} \| {party} \| {office} \| {state} \| {county}` | `Timothy Lance \| \| BOARD OF EDUCATION DISTRICT 02 \| NC \| Columbus` |
| Contest | `{raw_name} \| {office_level} \| {state} {year}` | `COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02 \| school_district \| NC 2022` |
| Geography | `{municipality}, {county} County, {state}` | `Whiteville, Columbus County, NC` |

Middle initials and suffixes are included deliberately. "David S Marshall | ME" and "David A Marshall | FL" produce different vectors — measured at cosine 0.6448 with middle initials versus 0.7025 without. That 0.058 gap is the difference between correct separation and a false merge. See [Composite String Templates](./embeddings-composites.md) for the full rationale, including the "context bleed" problem where shared geographic context artificially inflates similarity between unrelated candidates.

Empty components (null middle, null suffix) produce empty slots in the template rather than being omitted. This keeps the template structure consistent, which stabilizes the embedding model's tokenization.

## FAISS Indices

Embeddings are stored in partitioned FAISS indices, one per (state, year) combination. Partitioning serves two purposes:

1. **Blocking alignment.** Entity resolution at L3 blocks by `(state, office_level, last_name_initial)`. State-level FAISS partitions ensure that nearest-neighbor queries never cross state boundaries — a candidate in NC is never compared to a candidate in FL during retrieval.

2. **Memory management.** A single national index for 42 million candidate embeddings at 3,072 dimensions × 4 bytes = ~500 GB of float32 data. Per-state-year partitions fit in memory on commodity hardware. NC 2022 (~200K records × 3,072 dims × 4 bytes) is approximately 2.3 GB.

Index type is `IndexFlatIP` (inner product on L2-normalized vectors, equivalent to cosine similarity). No approximate search — exact cosine is computed for every candidate pair within a block. At partition scale, exact search is fast enough (sub-second for 200K vectors) and avoids the recall loss of approximate methods like IVF or HNSW.

## Tier 3 Office Classification

Records that were not classified by L1's keyword (tier 1) or regex (tier 2) classifiers are embedded and compared against a reference set of ~200 pre-classified office names.

The reference set is a curated list covering every `(office_level, office_branch)` pair with at least 3 examples. Each reference entry has a pre-computed embedding. For an unclassified office name, L2 computes its embedding, finds the nearest reference neighbor by cosine similarity, and assigns the reference's classification if the score exceeds 0.60.

Real tier 3 results:

| Unclassified Name | Nearest Reference | Cosine | Assigned Classification |
|-------------------|------------------|:------:|------------------------|
| Collier Mosquito Control District | Mosquito Control District | 0.787 | special_district / infrastructure |
| Eastern Carrituck Fire & Rescue | Fire Protection District | 0.724 | special_district / infrastructure |
| Lowndes County Bd of Ed | Board of Education | 0.831 | school_district / education |

Names scoring below 0.60 are left as `other` at L2 and passed to tier 4 (LLM) at L3. Tier 3 classifies approximately 4.5% of the unique office names that survived tiers 1 and 2, with 94% accuracy against manual review.

The classification result is written back into the L1-inherited fields on the enriched L2 record, updating `classifier_method` to `"embedding_nn"` and `classifier_confidence` to the cosine score.

## Quality Flags

L2 raises flags on records with characteristics that may cause downstream problems. Flags do not block processing — they annotate records for review at L4.

| Flag | Condition | Example |
|------|-----------|---------|
| `short_name` | Candidate name has ≤ 2 characters after decomposition | `"J. D."` with no last name parsed |
| `common_name_risk` | First + last name appears 50+ times nationally | `John Smith`, `Robert Johnson` |
| `missing_office_level` | Office survived all classification tiers as `other` | `Santa Rosa Island Authority` (pre-tier-4) |
| `zero_votes` | `votes_total` is 0 | Write-in candidates with no votes |
| `high_vote_share` | Single candidate has > 99% of votes in a contested race | Possible data error or unopposed misclassification |

In our prototype, 12 of 200 records received at least one quality flag. The most common was `zero_votes` (write-in placeholders), followed by `common_name_risk`.

## Output Format

L2 produces two types of output per (state, year) partition:

**Enriched JSONL** — L1 records augmented with an `l2` block:

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

**Binary sidecars** — `.npy` files containing float32 arrays of embeddings, plus a JSON ID mapping:

```text
l2_embedded/nc/2022/
├── enriched.jsonl                  # One record per line, all L1 + L2 fields
├── candidate_embeddings.npy        # float32[N, 3072]
├── contest_embeddings.npy          # float32[M, 3072]
├── geography_embeddings.npy        # float32[K, 3072]
└── id_mapping.json                 # l1_hash → embedding row index
```

Embeddings are stored separately from JSONL to keep the text records streamable. A 3,072-dimensional float32 vector is 12,288 bytes — embedding it as base64 inside JSON would triple the JSONL file size. The `.npy` format is readable by NumPy, PyTorch, and any tool that understands the NumPy array file specification.

The `candidate_embedding_id` in the JSONL record is an integer index into `candidate_embeddings.npy`. To retrieve Timothy Lance's embedding: load the `.npy` file, index row 4271.

## Determinism

L2 is deterministic given the same L1 input, the same embedding model version, and the same office reference set. The composite string templates are fixed. The FAISS index construction is deterministic (flat index, no random initialization). The tier 3 nearest-neighbor search is exact.

If OpenAI updates the weights behind `text-embedding-3-large` without changing the model name, the vectors change silently. The `embedding_model` field cannot detect this — it records the API model name, not an internal version hash. In practice, OpenAI has not changed embedding model weights after release. If they do, a full L2 re-run and threshold recalibration is required.

## Dependencies

L2 requires an OpenAI API key for embedding generation. L0 and L1 do not — they run entirely offline. This is the first layer that requires network access.

At prototype scale (200 records), L2 embedding generation takes approximately 3 seconds and costs less than $0.01. At production scale (42 million rows), the cost is approximately $300 and the wall-clock time depends on API throughput (typically 3,000 embeddings per minute with batching, yielding ~10 days for the full corpus). Embeddings are computed once per L1 record and cached — re-running L3 or L4 does not re-invoke the embedding API.