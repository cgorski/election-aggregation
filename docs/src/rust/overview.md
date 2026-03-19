# Crate Overview

The `election-aggregation` crate is both a Rust library (`election_aggregation`) and a command-line binary (`election-aggregation`). The library provides types, parsers, and pipeline logic. The binary provides the CLI entry point.

## Crate Configuration

From `Cargo.toml`:

| Field | Value |
|-------|-------|
| Edition | 2024 |
| `rust-version` | 1.93 |
| Library name | `election_aggregation` |
| Binary name | `election-aggregation` |
| License | MIT OR Apache-2.0 |

The library is published as `election_aggregation` (underscored, per Rust convention). The binary is `election-aggregation` (hyphenated, per CLI convention). Both are defined in the same crate.

## Module Structure

```text
src/
├── lib.rs              # Library root — re-exports all public modules
├── main.rs             # Binary entry point — CLI dispatch
├── schema/
│   └── mod.rs          # Unified record types, enums, and field definitions
├── sources/
│   ├── mod.rs          # Source registry and SourceParser trait
│   ├── medsl.rs        # MEDSL parser (25-column CSV/TSV)
│   ├── ncsbe.rs        # NC SBE parser (15-column tab-delimited)
│   ├── openelections.rs # OpenElections parser (variable CSV)
│   ├── clarity.rs      # Clarity/Scytl XML parser
│   ├── vest.rs         # VEST shapefile parser (column decoding)
│   ├── census.rs       # Census FIPS reference file loader
│   └── fec.rs          # FEC candidate master file parser
└── pipeline/
    ├── mod.rs          # Layer sequencing and orchestration
    ├── l0.rs           # Raw acquisition (byte-identical storage + manifest)
    ├── l1.rs           # Deterministic parsing and enrichment
    ├── l2.rs           # Embedding generation (text-embedding-3-large)
    ├── l3.rs           # Entity resolution (cascade: exact → Jaro-Winkler → embedding → LLM)
    └── l4.rs           # Canonical name assignment, temporal chains, verification
```

Three top-level modules, each with a clear responsibility:

- **`schema`** — Defines the unified record types that all sources normalize into. Contains `ContestKind`, `CandidateName`, `VoteCountsByType`, all enumerations, and the layer-specific record structs (`L1Record` through `L4Record`). No I/O, no parsing logic.

- **`sources`** — One submodule per data source. Each submodule documents the source schema, implements parsing from the source format into L1 records, and catalogs known data quality issues. The parent `mod.rs` defines the `SourceParser` trait that all sources implement.

- **`pipeline`** — One submodule per layer. Each layer reads its parent layer's JSONL output and writes its own. `l0` handles acquisition. `l1` calls into source parsers. `l2` batches embedding API calls. `l3` batches LLM calls. `l4` builds the entity graph.

## Library vs. Binary

The library (`src/lib.rs`) exposes three public modules:

```rust
pub mod sources;
pub mod pipeline;
pub mod schema;
```

External crates can depend on `election_aggregation` to use the types and parsers without the CLI. The binary (`src/main.rs`) imports the library and wires it to CLI argument parsing.

The current binary prints usage information and a pointer to the documentation. CLI subcommands (`process`, `embed`, `match`, `canonicalize`, `verify`, `sources`) are planned but not yet implemented — see [CLI Reference](./cli.md).

## Dependencies

The `Cargo.toml` currently declares no runtime dependencies. As pipeline layers are implemented, expected dependencies include:

| Crate | Purpose |
|-------|---------|
| `serde` + `serde_json` | JSONL serialization/deserialization |
| `csv` | CSV/TSV parsing for MEDSL, NC SBE, OpenElections |
| `sha2` | SHA-256 hashing for the provenance chain |
| `clap` | CLI argument parsing |
| `reqwest` | HTTP client for embedding and LLM API calls |
| `tokio` | Async runtime for batched API calls (L2, L3) |

The release profile enables LTO, single codegen unit, and symbol stripping for minimal binary size.

## Build

```sh
cargo build --release
./target/release/election-aggregation
```

Minimum supported Rust version is 1.93, matching edition 2024 requirements.