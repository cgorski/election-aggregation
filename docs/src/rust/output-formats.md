# Output Format: JSONL and CSV Export

The pipeline writes JSONL at every layer. JSONL is the canonical format — it is the source of truth for every record at every stage. L4 additionally exports flat CSV for spreadsheet users. Embedding vectors at L2 are stored as `.npy` binary sidecars alongside the JSONL.

## JSONL — Canonical at Every Layer

Every pipeline layer (L1 through L4) writes its output as JSONL: one JSON object per line, one file per state/year partition.

File naming convention:

```
{layer}/{state_po}/{year}.jsonl
```

Examples:

| Path | Contents |
|------|----------|
| `l1/NC/2022.jsonl` | All L1 cleaned records for North Carolina 2022 |
| `l2/NC/2022.jsonl` | L2 records with embedding metadata (vectors stored separately) |
| `l3/NC/2022.jsonl` | L3 records with entity resolution cluster IDs |
| `l4/NC/2022.jsonl` | L4 canonical records with verification status |

Properties:

- **One record per line.** Each line is a complete, self-contained JSON object. No multi-line formatting.
- **Streamable.** Consumers can process records one at a time without loading the full file into memory.
- **Appendable.** New records are concatenated to the end of the file. Existing lines are never modified.
- **Serialized with `serde_json`.** All Rust types implement `Serialize` and `Deserialize` via serde. Field names in JSON match the Rust struct field names exactly.

A single JSONL line for an L1 record contains all six schema sections (election, jurisdiction, contest, results, turnout, source, provenance) as top-level keys. Null fields are included explicitly rather than omitted, so every record has the same set of keys.

## Embedding Vectors — `.npy` Sidecars

Embedding vectors generated at L2 are not stored inside the JSONL records. A 3072-dimensional `f32` vector (text-embedding-3-large output) occupies 12,288 bytes — storing it as a JSON array of floats would roughly triple the file size per record.

Instead, vectors are written as NumPy `.npy` binary files alongside the JSONL:

| File | Contents |
|------|----------|
| `l2/NC/2022.jsonl` | L2 records with `embedding_model`, `embedding_version`, and vector array index |
| `l2/NC/2022_candidate_name.npy` | Dense matrix: one row per record, 3072 columns |
| `l2/NC/2022_contest_name.npy` | Dense matrix for contest name embeddings |
| `l2/NC/2022_jurisdiction.npy` | Dense matrix for jurisdiction embeddings |

Each JSONL record at L2 contains an `embedding_index` field (integer) that identifies which row of the `.npy` matrix corresponds to that record. The `.npy` format is a simple binary header followed by contiguous `f32` values — readable by NumPy, PyTorch, and any tool that understands the format.

The `.npy` files are written once and never modified. Re-embedding with a different model version produces new files with a version suffix (e.g., `2022_candidate_name_v2.npy`).

## CSV Export at L4

L4 produces a flat CSV in addition to JSONL. The CSV is designed for spreadsheet users and tools like pandas, R, or DuckDB that work with tabular data.

The CSV flattens the nested JSONL structure:

- `CandidateName` components become separate columns: `candidate_raw`, `candidate_first`, `candidate_middle`, `candidate_last`, `candidate_suffix`, `candidate_nickname`.
- `VoteCountsByType` becomes: `votes_election_day`, `votes_early`, `votes_absentee_mail`, `votes_provisional`.
- Nested objects (election, jurisdiction, contest, source, provenance) are flattened with underscore-separated prefixes.
- The `results` array is denormalized: one CSV row per candidate per precinct per contest (matching the JSONL structure, which already stores one result per record after L1 normalization).

The CSV omits embedding vectors, raw source fields, and hash chain details. These are available in the JSONL for users who need them.

## Design Rationale

**Why JSONL over Parquet or SQLite?** JSONL is human-readable, appendable, and requires no special tooling to inspect (`head`, `jq`, `grep` all work). It supports the nested schema (CandidateName, VoteCountsByType, SourceRawFields) without flattening. The tradeoff is file size and query performance — both are addressed by the L4 CSV export and by the fact that consumers can convert JSONL to Parquet with a one-liner (`duckdb "COPY (SELECT * FROM read_json('l4/NC/2022.jsonl')) TO 'l4/NC/2022.parquet'"`).

**Why `.npy` over embedding in JSON?** Size. A 42M-record corpus with three 3072-dimensional vectors per record would produce ~1.5 TB of JSON-encoded floats. The `.npy` binary format stores the same data in ~460 GB with zero parsing overhead.

**Why CSV at L4 only?** L1–L3 records contain fields (embedding indices, match method metadata, hash chains) that do not map to a flat table. L4 is the consumer-facing layer where the schema is stable enough for tabular export.