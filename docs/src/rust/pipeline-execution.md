# Pipeline Execution

The pipeline processes records through five layers in strict order: L0 → L1 → L2 → L3 → L4. Each layer reads its parent's JSONL output and writes its own. No layer skips its predecessor.

## Streaming Processing

Records are processed one at a time. The pipeline never loads an entire layer's output into memory. Each layer reads a line from its input JSONL, transforms it, and writes a line to its output JSONL. This keeps memory usage proportional to a single record, not to the dataset size.

For a 42M-row corpus, this is not optional. Loading 12.3M MEDSL 2022 rows into memory as deserialized structs would require tens of gigabytes. Streaming keeps the resident set under 500 MB for L0 → L1 and L1 → L2.

## Partitioning

All processing is partitioned by **state and year**. Each partition is an independent unit of work:

```
l1/NC/2022/medsl.jsonl
l1/NC/2022/ncsbe.jsonl
l1/FL/2022/medsl.jsonl
l1/FL/2022/openelections.jsonl
```

Partitioning enables:

- **Incremental processing.** Re-running L1 for North Carolina does not require re-processing Texas.
- **Parallelism.** Independent partitions can be processed concurrently.
- **Bounded working sets.** L4's entity graph (which does require in-memory state) is scoped to one state-year at a time rather than the full corpus.

## Layer-Specific Execution

### L0 → L1: Deterministic, Single-Record

Each source row is parsed independently. No row depends on any other row. This is purely CPU-bound — no network calls, no model inference. On a single core, L1 processes approximately 200,000 MEDSL rows per second.

### L1 → L2: Batched Embedding API Calls

L2 generates embeddings using `text-embedding-3-large`. The OpenAI embedding API accepts batches of up to 2,048 inputs per request. L2 accumulates records into batches of 256 (configurable), constructs composite strings from name components and contest fields, sends the batch to the API, and attaches the returned vectors to each record.

Batching amortizes HTTP overhead. At 256 records per batch, a 12.3M-row state-year partition requires approximately 48,000 API calls. Rate limiting and retry logic are handled at this layer.

Embedding vectors are written as `.npy` binary sidecar files, not inline in JSONL. The JSONL record carries a reference (file path + offset) to the corresponding vector. This keeps JSONL files human-readable and text-diffable.

### L2 → L3: Batched LLM Calls

L3 performs entity resolution in three tiers. The first tier (deterministic blocking) and second tier (embedding nearest-neighbor) require no API calls. The third tier sends ambiguous candidate pairs to Claude Sonnet for confirmation.

LLM calls are batched per contest cluster — all ambiguous pairs within a single contest are sent in one structured prompt. This reduces call count and provides the LLM with full context (all candidates, all name variants, the office title, the jurisdiction).

The deterministic tier resolves 70%+ of records. The embedding tier resolves most of the remainder. LLM calls are made for approximately 5–10% of entity resolution decisions, concentrated on cases where name similarity is 0.85–0.92.

### L3 → L4: In-Memory Entity Graph

L4 is the exception to the streaming rule. Building temporal chains (linking the same candidate across election cycles) and selecting canonical names requires the full entity graph for a partition in memory. For a single state, this graph typically contains 10,000–50,000 entity nodes.

L4 loads all L3 records for one state-year partition, constructs the candidate and contest entity graphs, assigns canonical names, builds temporal chain links, runs verification checks against the hash chain, and writes the final L4 JSONL and CSV outputs.

Memory usage scales with the number of unique entities in a partition, not the number of rows. North Carolina (the largest single-state partition due to NC SBE's 10 cycles) peaks at approximately 2 GB for the entity graph.

## Error Handling

Each layer writes a **quarantine log** alongside its output JSONL. Records that fail parsing, embedding, or matching are written to the quarantine file with a structured error message. They do not block processing of subsequent records.

Quarantine files follow the naming convention:

```
l1/NC/2022/medsl.quarantine.jsonl
```

Each quarantine entry contains the original record (or as much as could be parsed), the error type, and the error message. Quarantine rates by layer:

| Layer | Typical quarantine rate | Common causes |
|-------|------------------------|---------------|
| L1 | 0.1% | Non-integer vote values, unparseable names, encoding errors |
| L2 | <0.01% | API timeouts (retried), embedding dimension mismatch |
| L3 | 1–3% | Ambiguous matches below confidence threshold |
| L4 | <0.1% | Hash chain verification failures |