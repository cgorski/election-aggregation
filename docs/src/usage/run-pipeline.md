# Run the Pipeline

> **Note:** The CLI described in this chapter is the planned interface. It is not yet implemented. This documents the target design so that the architecture, schema, and documentation are aligned before code is written.

## Layer-by-Layer Execution

Each layer reads the output of the previous layer and produces JSONL. Layers are run independently — if L2 fails, fix the issue and re-run L2 without re-running L0 or L1.

### L0 → L1: Parse and Clean

```text
election-aggregation process \
  --source ncsbe \
  --input local-data/sources/nc_sbe/results_pct_20221108.txt \
  --output local-data/processed/l1_cleaned/nc_sbe/NC/2022/
```

No API keys required. Produces `cleaned.jsonl` and `cleaning_report.json`. The cleaning report lists records routed to `TurnoutMetadata`, `BallotMeasure`, and any rows that failed parsing.

### L1 → L2: Embed

```text
election-aggregation embed \
  --input local-data/processed/l1_cleaned/nc_sbe/NC/2022/cleaned.jsonl \
  --output local-data/processed/l2_embedded/NC/2022/
```

Requires `OPENAI_API_KEY`. Produces `enriched.jsonl`, `candidate_embeddings.npy`, `contest_embeddings.npy`, and `id_mapping.json`. Also runs tier 3 office classification against the reference set.

### L2 → L3: Match Entities

```text
election-aggregation match \
  --input local-data/processed/l2_embedded/NC/2022/ \
  --output local-data/processed/l3_matched/NC/2022/
```

Requires `ANTHROPIC_API_KEY`. Produces `matched.jsonl` and `decisions/candidate_matches.jsonl`. The decision log records every comparison — exact matches, gate rejections, embedding auto-accepts, and LLM calls with full prompts and responses.

### L3 → L4: Canonicalize and Verify

```text
election-aggregation canonicalize \
  --input local-data/processed/l3_matched/NC/2022/ \
  --output local-data/processed/l4_canonical/
```

Requires `ANTHROPIC_API_KEY` for the LLM entity audit. Produces `candidate_registry.json`, `contest_registry.json`, `verification_report.json`, and `exports/flat_export.jsonl`.

## Re-Running Individual Layers

Each layer reads only its predecessor's output. To re-run L2 with a different embedding model:

```text
election-aggregation embed \
  --input local-data/processed/l1_cleaned/nc_sbe/NC/2022/cleaned.jsonl \
  --output local-data/processed/l2_embedded_v2/NC/2022/ \
  --model text-embedding-3-small
```

L1 output is untouched. L3 and L4 must be re-run against the new L2 output, and thresholds must be recalibrated for the new model.

## Troubleshooting

If a step fails, check:

- **L1 failure** → `cleaning_report.json` lists unparseable rows with line numbers and error messages.
- **L2 failure** → Usually an API key issue or rate limit. The embed command is resumable — it skips records that already have embeddings in the output directory.
- **L3 failure** → The decision log (`candidate_matches.jsonl`) records progress. Re-running skips already-decided pairs (replay from log).
- **L4 failure** → The verification report identifies which algorithm failed and on which records.