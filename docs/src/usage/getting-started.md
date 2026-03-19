# Getting Started

This chapter describes the planned interface for running the election-aggregation pipeline. The CLI is not yet implemented — this documents the target design so that early users can understand the workflow and contributors can build toward it.

## Prerequisites

| Requirement | Version | Purpose |
|-------------|---------|---------|
| Rust toolchain | 1.93+ | Build and run the pipeline |
| Disk space | 8 GB minimum | Raw source files + processed output |
| OpenAI API key | — | L2 embedding generation (`text-embedding-3-large`) |
| Anthropic API key | — | L3 entity resolution and L4 entity audit (Claude Sonnet) |

L0 and L1 require no API keys. You can download data and run deterministic parsing without any external service. L2 requires OpenAI. L3 requires Anthropic. L4 verification re-uses the Anthropic key for the entity audit step.

## Install

Clone the repository and build:

```text
git clone https://github.com/your-org/election-aggregation.git
cd election-aggregation
cargo build --release
```

Or install directly:

```text
cargo install --path .
```

The binary is `election-aggregation`. Verify with:

```text
election-aggregation --version
```

## API Key Configuration

Set environment variables for the layers that require them:

```text
export OPENAI_API_KEY="sk-..."        # Required for L2
export ANTHROPIC_API_KEY="sk-ant-..."  # Required for L3 and L4
```

Keys are never stored in configuration files, command history, or pipeline output. The pipeline reads them from the environment at invocation time.

## Quick Start

The minimal workflow downloads NC SBE 2022 data and runs L0 through L1 — no API keys needed:

```text
# Download NC SBE 2022 general election results
election-aggregation download --source ncsbe --year 2022

# Process L0 → L1 (deterministic, offline)
election-aggregation process --source ncsbe --year 2022
```

This produces JSONL output at `local-data/processed/l1_cleaned/nc_sbe/NC/2022/cleaned.jsonl`. You can query it immediately with `jq` or Python. See [Querying JSONL Output](./query-jsonl.md).

To continue through the full pipeline:

```text
# L1 → L2 (requires OpenAI key)
election-aggregation embed --state NC --year 2022

# L2 → L3 (requires Anthropic key)
election-aggregation match --state NC --year 2022

# L3 → L4 (deterministic construction + LLM audit)
election-aggregation canonicalize --state NC --year 2022
```

Each layer reads the prior layer's output and writes to the next layer's directory. If a step fails, check the cleaning report (`cleaning_report.json` at L1) or the decision log (`candidate_matches.jsonl` at L3) for diagnostics.

## Re-Running Individual Layers

Layers are independent. Re-running L2 does not require re-running L1 — it reads from existing L1 output. Re-running L3 does not require re-running L2. This means:

- If you upgrade the embedding model, re-run L2 and everything downstream (L3, L4).
- If you add a nickname to the dictionary, re-run L1 and everything downstream (L2, L3, L4).
- If you override an L3 entity match decision, re-run L4 only.

## What Is Not Yet Implemented

The CLI commands above describe the planned interface. As of the current version, the pipeline runs through Rust library code and test harnesses, not a polished CLI. The following are planned but not yet available:

- `election-aggregation download` — automated source fetching with hash verification
- `election-aggregation process` — L0→L1 pipeline with progress reporting
- `election-aggregation embed` — L1→L2 with batched API calls and resume-on-failure
- `election-aggregation match` — L2→L3 with configurable thresholds and replay mode
- `election-aggregation canonicalize` — L3→L4 with verification report generation
- CSV export from L4

Contributions are welcome. See [Crate Overview](../rust/overview.md) for the current code structure.