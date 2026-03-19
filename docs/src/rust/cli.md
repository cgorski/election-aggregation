# CLI Reference

The `election-aggregation` binary provides a command-line interface for pipeline execution and data source management. Commands are not yet implemented — this chapter documents the planned interface.

## Planned Commands

| Command | Pipeline stage | Description |
|---------|---------------|-------------|
| `election-aggregation process` | L0 → L1 | Parse raw source files into cleaned JSONL records |
| `election-aggregation embed` | L1 → L2 | Generate text-embedding-3-large vectors for candidate names, contest names, and jurisdictions |
| `election-aggregation match` | L2 → L3 | Run entity resolution: exact → Jaro-Winkler → embedding → LLM confirmation |
| `election-aggregation canonicalize` | L3 → L4 | Assign canonical names, build temporal chains, produce verification status |
| `election-aggregation verify` | L4 | Walk the hash chain from L4 back to L0 source bytes and report any breaks |
| `election-aggregation sources` | — | List all data sources with download URLs and instructions |

## Common Options

All pipeline commands will accept:

- `--state <STATE>` — Process a single state (two-letter postal code). Without this flag, all states are processed.
- `--year <YEAR>` — Process a single election year. Without this flag, all loaded years are processed.
- `--data-dir <PATH>` — Root directory for source files and pipeline output. Defaults to `./local-data`.
- `--jobs <N>` — Number of parallel state/year partitions to process. Defaults to 1.

## API Key Configuration

L2 (`embed`) requires an OpenAI API key for text-embedding-3-large. L3 (`match`) requires an Anthropic API key for Claude Sonnet confirmation calls. Keys are read from environment variables:

```
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
```

The `process` and `canonicalize` commands do not call external APIs.

## Implementation Status

The binary currently prints a version banner and documentation pointer. No subcommands are wired up. The CLI will use `clap` for argument parsing once pipeline modules are functional.