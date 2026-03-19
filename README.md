# Election Aggregation

**A multi-layer pipeline for collecting, normalizing, and unifying US local election results from heterogeneous sources.**

📖 **[Read the full documentation →](https://cgorski.github.io/election-aggregation/)**

---

## The problem

There is no national database of US local election results. The data exists — scattered across 50 state election boards, 3,000+ county clerk offices, academic datasets, and election night reporting platforms — but it has never been unified into a single, consistent, trustworthy format.

Every source uses different schemas, different name formats, different office titles, and different levels of completeness. The same candidate appears as `SHANNON W BRAY` in one source and `Shannon W. Bray` in another. The same office means different things in different states — a "County Judge" in Texas is a chief executive, not a judicial officer. And nobody tracks the same person across elections, so questions like "has my county commissioner been reelected five times unopposed?" are unanswerable.

This project fixes that.

## What this project does

Election Aggregation processes election data through five immutable layers:

```text
L0  RAW        → Byte-identical source files with acquisition manifests
L1  CLEANED    → Parsed records with all name components preserved (deterministic, no ML)
L2  EMBEDDED   → Vector embeddings for fuzzy matching and classification
L3  MATCHED    → Entity-resolved records with candidate/contest identifiers
L4  CANONICAL  → Authoritative names, temporal chains, verification, exports
```

The ordering is strict: **Clean → Embed → Match → Canonicalize.** You cannot assign an authoritative name before you know who the person is. Every record carries a hash chain back to the original source bytes. Every entity resolution decision is logged with full reasoning.

## What this project does not do

- **It does not store election data.** Data files are large and published by their respective sources. This project tells you where to get the data, documents every source schema, and provides tools to process it. You download the data yourself.
- **It is not a real-time election tracker.** We process official/certified results, not live feeds.
- **It does not claim perfect accuracy.** Entity resolution is probabilistic. Known limitations are [documented](https://cgorski.github.io/election-aggregation/trust/limitations.html). Every match decision is auditable.

## What you can answer

With data from MEDSL (all 50 states, 2018–2022), NC SBE (2006–2024), OpenElections (6 states), VEST (shapefiles), Census FIPS, and FEC:

| Question | Example finding |
|----------|-----------------|
| How many sheriffs ran unopposed? | 55% in NC, 77% in Maine |
| Closest school board race in America? | Dawson County, GA — exact tie, 25,186 to 25,186 |
| What percentage of local races are uncontested? | 48.8% nationally |
| Which office type is least competitive? | Constable/Coroner at 72% uncontested |
| Who has served longest on a local body in NC? | George Dunlap — Mecklenburg County Commissioner, 6 cycles (2014–2024) |
| How many unique elected offices exist? | 8,387+ distinct names; 4,995 exist in exactly one county |

See the [full list of answerable and not-yet-answerable questions](https://cgorski.github.io/election-aggregation/introduction.html#what-you-can-answer-today) in the documentation.

## Quick start

### Prerequisites

- Rust 1.93+ (edition 2024)
- ~8 GB disk space for source data
- OpenAI API key (for L2 embeddings — `text-embedding-3-large`)
- Anthropic API key (for L3 entity resolution)

### Install

```sh
cargo install --path .
```

### Download data

This project does not ship with data. It tells you where to get data, documents every source schema, and provides tools to process it. Download sources following the instructions in the [data sources documentation](https://cgorski.github.io/election-aggregation/sources/overview.html), or use the quick-start script:

```sh
# MEDSL 2022 — all 50 states (precinct-level, ~2 GB compressed)
mkdir -p local-data/sources/medsl/2022
for state in ak al ar az ca co ct dc de fl ga hi ia id il in ks ky la ma md me mi mn mo ms mt nc nd ne nh nj nm nv ny oh ok or pa ri sc sd tn tx ut va vt wa wi wv wy; do
  curl -L -o "local-data/sources/medsl/2022/2022-${state}-local-precinct-general.zip" \
    "https://raw.githubusercontent.com/MEDSL/2022-elections-official/main/individual_states/2022-${state}-local-precinct-general.zip"
done

# NC SBE 2022 — gold standard with vote mode breakdowns
mkdir -p local-data/sources/ncsbe/2022
curl -L -o local-data/sources/ncsbe/2022/results_pct_20221108.zip \
  "https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2022_11_08/results_pct_20221108.zip"
```

See [Download Instructions](https://cgorski.github.io/election-aggregation/usage/download.html) for the full set of sources including MEDSL 2018/2020, NC SBE 2006–2024, OpenElections, VEST shapefiles, Census FIPS, and FEC candidate files.

### Run the pipeline

```sh
# Process NC SBE 2022 through L0→L1 (deterministic, no API keys needed)
election-aggregation process --source ncsbe --input local-data/sources/ncsbe/2022/ --output local-data/processed/

# Process through L2 (requires OpenAI API key for embeddings)
election-aggregation embed --input local-data/processed/l1/ --output local-data/processed/l2/

# Process through L3 (requires Anthropic API key for entity resolution)
election-aggregation match --input local-data/processed/l2/ --output local-data/processed/l3/

# Produce L4 canonical output with verification
election-aggregation canonicalize --input local-data/processed/l3/ --output local-data/processed/l4/
```

### Pipeline output format

Every layer stores its output as **JSONL (JSON Lines)** — one JSON record per line, in directory hierarchies organized by source/state/year. JSONL is the canonical format at every layer. L4 also produces a flat CSV export for spreadsheet users.

```text
local-data/processed/
├── l0_raw/{source}/{file}.jsonl          # Byte-identical source copies + .manifest.json
├── l1_cleaned/{source}/{state}/{year}/   # Parsed records, unified schema (JSONL)
├── l2_embedded/{state}/{year}/           # Enriched records (JSONL) + embedding sidecars (.npy)
├── l3_matched/{state}/{year}/            # Entity-resolved records (JSONL) + decision logs (JSONL)
└── l4_canonical/
    ├── candidate_registry.json           # All resolved person entities
    ├── contest_registry.json             # All resolved contest entities
    ├── verification_report.json          # Hash chain + entity audit results
    └── exports/
        ├── flat_export.jsonl             # Researcher-facing flattened records
        └── flat_export.csv               # Same data as CSV for spreadsheet users
```

The `.npy` sidecar files at L2 hold embedding vectors (binary float32 arrays) for FAISS retrieval. These are internal to the pipeline and not consumer-facing.

### Query the output

The JSONL output can be queried directly with any tool that reads JSON — `jq`, Python, or loaded into an analytical engine for SQL access:

```sh
# With jq — find close races
cat local-data/processed/l4_canonical/exports/flat_export.jsonl \
  | jq -r 'select(.vote_share != null and .vote_share > 0.48 and .vote_share < 0.52) | [.state, .contest_name, .candidate_canonical, .vote_share] | @tsv'

# With Python
python -c "
import json
for line in open('local-data/processed/l4_canonical/exports/flat_export.jsonl'):
    r = json.loads(line)
    if r.get('office_level') == 'school_district':
        print(f\"{r['state']} | {r['contest_name']} | {r['candidate_canonical']} | {r['votes_total']}\")
"
```

> **Note:** During development we used [DuckDB](https://duckdb.org) to run ad-hoc SQL over raw source CSVs for exploratory analysis. DuckDB is a convenient tool for querying large JSONL or CSV files with SQL, but it is not part of the pipeline architecture. The pipeline's source of truth is always the JSONL files in the layer directories.

See [Recipes](https://cgorski.github.io/election-aggregation/usage/recipes.html) for ready-to-use queries against the pipeline output.

## Data sources

We process data from these sources. We do not store or redistribute their data.

| Source | Coverage | Format | Docs |
|--------|----------|--------|------|
| [MEDSL](https://dataverse.harvard.edu/dataverse/electionscience) | 50 states + DC, 2018/2020/2022 | CSV (25 cols) | [→](https://cgorski.github.io/election-aggregation/sources/medsl.html) |
| [NC SBE](https://www.ncsbe.gov/results-data) | NC, 2006–2024 | TSV (15 cols) | [→](https://cgorski.github.io/election-aggregation/sources/ncsbe.html) |
| [OpenElections](https://openelections.net) | ~8 states, varies | CSV (7+ cols) | [→](https://cgorski.github.io/election-aggregation/sources/openelections.html) |
| [Clarity/Scytl](https://results.enr.clarityelections.com) | ~1,000+ jurisdictions | XML | [→](https://cgorski.github.io/election-aggregation/sources/clarity.html) |
| [VEST](https://dataverse.harvard.edu/dataverse/electionscience) | 50 states (shapefiles) | SHP/DBF | [→](https://cgorski.github.io/election-aggregation/sources/vest.html) |
| [Census Bureau](https://www2.census.gov/geo/docs/reference/codes2020/) | National FIPS reference | TXT | [→](https://cgorski.github.io/election-aggregation/sources/census.html) |
| [FEC](https://www.fec.gov/data/browse-data/?tab=bulk-data) | Federal candidates | TXT | [→](https://cgorski.github.io/election-aggregation/sources/fec.html) |

## Storage format

**JSONL (JSON Lines) is the canonical format at every pipeline layer.** One JSON record per line, stored in directory hierarchies organized by source, state, and year. This format is:

- **Human-readable** — open any file in a text editor and see structured records
- **Streamable** — process line-by-line without loading everything into memory
- **Appendable** — new records can be added without rewriting the file
- **Tool-agnostic** — works with `jq`, Python, Rust `serde_json`, or any language

L4 additionally exports a flat CSV for spreadsheet users. Embedding vectors at L2 are stored as `.npy` (binary float32 arrays) sidecars alongside the JSONL — these are internal to the pipeline, not consumer-facing.

## Architecture at a glance

The core design principle is **deterministic first, embeddings for retrieval, LLMs for confirmation only.**

- **L0→L1** is purely deterministic. Same input always produces the same output. No ML, no API calls. A researcher can reproduce L1 on any machine.
- **L1→L2** is deterministic given the same embedding model version (`text-embedding-3-large`, 3072 dimensions).
- **L2→L3** involves LLM calls for ambiguous entity matches. Every decision is stored with the full prompt, response, reasoning, and token cost. Decisions can be replayed deterministically from the log.
- **L3→L4** is deterministic given L3. Canonical name selection follows a fixed algorithm (most complete name from most authoritative source).

Every record at every layer carries a SHA-256 hash chain back to L0. A [verification algorithm](https://cgorski.github.io/election-aggregation/architecture/provenance.html) at L4 can trace any claim — "Timothy Lance won 303 votes in precinct P17" — through every layer's JSONL files to the original NC SBE tab-delimited text file.

For the full architecture, read [Design Principles](https://cgorski.github.io/election-aggregation/architecture/principles.html) and [The Five-Layer Pipeline](https://cgorski.github.io/election-aggregation/architecture/pipeline-overview.html).

## The hard problems

What makes this project interesting (and difficult) is documented in detail:

- **[Name Normalization](https://cgorski.github.io/election-aggregation/hard-problems/name-normalization.html)** — `MICHAEL "STEVE" HUBER` vs `Michael (Steve) Huber` vs `M. Steve Huber`
- **[Entity Resolution](https://cgorski.github.io/election-aggregation/hard-problems/entity-resolution.html)** — Is `Charlie Crist` the same person as `CRIST, CHARLES JOSEPH`? (Yes — embedding score 0.451, LLM confidence 0.95)
- **[Office Classification](https://cgorski.github.io/election-aggregation/hard-problems/office-classification.html)** — 8,387 unique office names across 50 states, including "County Judge" which means different things in Texas vs everywhere else
- **[Non-Candidate Records](https://cgorski.github.io/election-aggregation/hard-problems/non-candidates.html)** — "Registered Voters", "BLANK", "TOTAL VOTES", "OverVotes" hiding inside candidate result files
- **[Cross-Source Reconciliation](https://cgorski.github.io/election-aggregation/hard-problems/cross-source.html)** — MEDSL and NC SBE overlap on 640 NC contests; 90.5% have exactly matching vote totals, 63% have different candidate name formatting

## Project structure

```text
election-aggregation/
├── Cargo.toml                 # Rust project (edition 2024, rust-version 1.93)
├── src/
│   ├── main.rs                # CLI entry point
│   ├── lib.rs                 # Library root
│   ├── sources/               # One module per data source
│   │   ├── medsl.rs
│   │   ├── ncsbe.rs
│   │   ├── openelections.rs
│   │   ├── clarity.rs
│   │   ├── vest.rs
│   │   ├── census.rs
│   │   └── fec.rs
│   ├── pipeline/              # Five-layer processing pipeline
│   │   ├── l0.rs              # Raw acquisition
│   │   ├── l1.rs              # Deterministic cleaning
│   │   ├── l2.rs              # Embedding generation
│   │   ├── l3.rs              # Entity resolution
│   │   └── l4.rs              # Canonicalization + verification
│   └── schema/                # Unified record types
├── docs/                      # mdbook documentation (the spec)
│   ├── book.toml
│   └── src/
│       ├── SUMMARY.md         # Table of contents
│       ├── introduction.md    # Start here
│   └── ...                # ~165 chapter files
├── local-data/                # NOT IN REPO — .gitignored
│   ├── README.md              # Data source docs and download instructions
│   ├── sources/               # Raw downloaded files (~8 GB of CSV/TSV/SHP)
│   └── processed/             # Pipeline output — JSONL at every layer (L0–L4)
├── legacy/                    # Previous Python prototype and design docs
└── tmp/                       # Experiment scratch files — .gitignored
```

> **Why JSONL?** Every pipeline layer writes JSONL — one self-contained JSON record per line. This is deliberate: JSONL is human-readable, streamable, appendable, and works with any language or tool. It avoids lock-in to any specific database or binary format. The Rust pipeline reads and writes JSONL natively via `serde_json`. Consumers can process it with `jq`, Python, R, or load it into any database they prefer.
```

## Documentation

The [mdbook documentation](https://cgorski.github.io/election-aggregation/) is both the user guide and the implementation spec. It covers:

- **[Part I: The Problem](https://cgorski.github.io/election-aggregation/problem/why-hard.html)** — Why local election data is fragmented and what questions we're trying to answer
- **[Part II: Data Sources](https://cgorski.github.io/election-aggregation/sources/overview.html)** — Every source: schema, download commands, quirks, coverage gaps
- **[Part III: The Hard Problems](https://cgorski.github.io/election-aggregation/hard-problems/name-normalization.html)** — Name normalization, entity resolution, office classification with real examples
- **[Part IV: Architecture](https://cgorski.github.io/election-aggregation/architecture/principles.html)** — The five-layer pipeline, hash chains, embedding strategy, LLM integration
- **[Part V: Unified Schema](https://cgorski.github.io/election-aggregation/schema/overview.html)** — The record format, field by field
- **[Part VI: Rust Implementation](https://cgorski.github.io/election-aggregation/rust/overview.html)** — Types, traits, module structure
- **[Part VII: Using the Data](https://cgorski.github.io/election-aggregation/usage/getting-started.html)** — Download, process, query JSONL output, with copy-paste recipes
- **[Part VIII: Trust and Reproducibility](https://cgorski.github.io/election-aggregation/trust/two-audiences.html)** — Confidence levels, error reporting, known limitations

Build and serve the docs locally:

```sh
cd docs && mdbook serve --open
```

## Trust model

There are two audiences with different trust needs:

**Engineers** (building or auditing the pipeline) see hash chains, layer manifests, LLM decision logs, embedding vector IDs, and can verify any record from L4 back to the raw source bytes.

**Consumers** (journalists, researchers, government staff) see source names, confidence levels, citations, and methodology documentation. They never see hashes, embeddings, or layer numbers. Their trust interface is: *"This number came from NC State Board of Elections certified data, retrieved March 2026."*

See [The Two Audiences](https://cgorski.github.io/election-aggregation/trust/two-audiences.html) for the full trust model.

## Known limitations

We are honest about what we cannot do yet:

- **Seven states** (CA, IA, KS, NJ, PA, TN, WI) have zero local race coverage in MEDSL 2022
- **Turnout data** exists in less than 5% of records — most sources don't report it
- **Odd-year elections** (2015, 2017, 2019, 2021) are underrepresented in our current data load
- **Entity resolution is probabilistic** — match decisions can be wrong; the decision log documents every judgment
- **Ranked-choice voting** is not yet supported in the schema
- **ALGED** (the only local-election-focused academic database) is not yet integrated due to access issues

See [Known Limitations](https://cgorski.github.io/election-aggregation/trust/limitations.html) for the complete list.

## Contributing

This project is in early development. The mdbook documentation is the spec — if you want to understand what needs to be built, read the book. If you want to contribute:

1. **Add a data source parser** — Implement the `SourceParser` trait for a new source. See [The SourceParser Trait](https://cgorski.github.io/election-aggregation/rust/source-parser-trait.html).
2. **Improve office classification** — Add keywords to the tier 1 classifier or regex patterns to tier 2. See [The Four-Tier Classifier](https://cgorski.github.io/election-aggregation/hard-problems/office-four-tiers.html).
3. **Expand the nickname dictionary** — Add name mappings. See [The Nickname Dictionary](https://cgorski.github.io/election-aggregation/hard-problems/names-dictionary.html).
4. **Report data quality issues** — If you find bad data, wrong entity matches, or misclassified offices, open an issue with the specific records.
5. **Write recipes** — Add queries that answer interesting questions using `jq`, Python, or other tools against the JSONL output. See [Recipes](https://cgorski.github.io/election-aggregation/usage/recipes.html).

## License

Code: MIT OR Apache-2.0

Documentation: CC-BY 4.0

This project does not store or redistribute election data. The data it processes is published by its respective sources under their own licenses (generally CC-BY or public domain).