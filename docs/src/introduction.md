# Election Aggregation

**A multi-layer pipeline for collecting, normalizing, and unifying US local election results from heterogeneous sources.**

---

## The question this project answers

Who ran for school board in your county last year? Who was the sheriff, and did anyone run against them? What was the closest local race in your state? Has your county commissioner been reelected five times unopposed, or do they face real competition?

These should be easy questions. They are not.

There is no national database of US local election results. The data exists — scattered across 50 state election boards, 3,000+ county clerk offices, academic datasets, election night reporting platforms, and community-curated repositories — but it has never been unified into a single, consistent, trustworthy format. Every source uses different schemas, different name formats, different office titles, different geographic identifiers, and different levels of completeness.

This project fixes that.

## What we found when we tried

We downloaded 42 million rows of precinct-level election data from MIT's Election Data Lab (MEDSL), the North Carolina State Board of Elections, OpenElections, VEST, the Census Bureau, and the FEC. We covered all 50 states across three election cycles (2018, 2020, 2022) and ten years of deep North Carolina history (2006–2024).

Then we tried to answer simple questions, and the problems started immediately.

**The same candidate appears differently across sources.** MEDSL reports `SHANNON W BRAY`. NC SBE reports `Shannon W. Bray`. One is all caps with no period after the middle initial. The other is title case with a period. These are the same person — but a computer doesn't know that without being told.

**Nicknames break everything.** `Charlie Crist` in one source is `CRIST, CHARLES JOSEPH` in another. A human recognizes Charlie as a nickname for Charles. An embedding model scores their similarity at 0.451 — well below any reasonable match threshold. A language model, given the context (same state, same office, same election, same vote count), correctly identifies them as the same person with 0.95 confidence.

**The same office title means different things.** In Texas, the "County Judge" is the chief executive of the county — equivalent to a county manager. In every other state, a county judge is a judicial officer. If your system classifies "DALLAS COUNTY JUDGE" as `judicial`, you're wrong in Texas and right everywhere else. Across all 50 states in 2022, we found **8,387 unique local office names**. Our keyword classifier handles 62% of them. The remaining 38% require embedding-based matching and LLM reasoning.

**Non-candidate data hides inside candidate data.** Florida OpenElections includes 6,013 rows labeled "Registered Voters" — not a contest, but a turnout metadata row that got slurped into the results file as if it were a race. Other sources include "BLANK" (Maine's name for undervotes), "TOTAL VOTES" (Utah's aggregation rows), and "OverVotes" / "UnderVotes" masquerading as candidate names. Each source has its own ghosts.

**Nobody tracks the same person across elections.** Timothy Lance won a Columbus County, NC school board seat in 2022. Did he run before? Did he win? Is the "T. Lance" who ran in 2018 the same person? No existing dataset answers this. Entity resolution — determining that two records refer to the same human being — is the hardest problem in this project, and the one we spend the most effort on.

## What this project does

Election Aggregation is a five-layer pipeline that transforms messy, heterogeneous election data into a clean, unified, entity-resolved dataset with full provenance back to the original source files.

```text
L0  RAW          Byte-identical source files. Never modified.
 ↓
L1  CLEANED      Parsed, structured records. Names decomposed into
                 first/middle/last/suffix. FIPS codes enriched.
                 Office classified by keyword and regex.
                 Purely deterministic. No ML, no API calls.
 ↓
L2  EMBEDDED     Vector embeddings generated for candidates, contests,
                 and geographic names. Office classification tier 3
                 (embedding nearest-neighbor). Quality flags raised.
                 Deterministic given the same embedding model.
 ↓
L3  MATCHED      Entity resolution. Same-candidate and same-contest
                 identifiers assigned. Embedding retrieval + LLM
                 confirmation. Every decision stored with reasoning.
 ↓
L4  CANONICAL    Authoritative names chosen. Temporal chains built.
                 Alias tables constructed. Verification algorithms run.
                 Researcher-facing exports produced.
```

The ordering is strict and deliberate: **Clean → Embed → Match → Canonicalize.** You cannot assign an authoritative name before you know who the person is. You cannot match entities before you have embeddings. You cannot embed before you have clean, signal-preserving parsed records. And you cannot parse before you have the raw bytes.

Every record at every layer carries a cryptographic hash chain back to the original source file. If someone modifies a vote count, changes a name, or alters a match decision at any layer, the verification algorithm detects exactly where the chain breaks.

## What this project does not do

- **It does not store election data.** The data files are large (7+ GB for our current corpus) and are published by their respective sources under their own terms. This project tells you where to get the data, documents every source's schema and quirks, and provides the tools to process it. You download the data yourself.

- **It is not a real-time election night tracker.** We ingest official and certified results, not live feeds. The pipeline is designed for post-election analysis, not real-time reporting.

- **It is not a prediction model.** We report what happened, not what will happen.

- **It does not claim perfect accuracy.** Entity resolution is probabilistic. Office classification has a 0.56% "other" rate. Some records have data quality issues we haven't caught yet. We document every known limitation, and every match decision is auditable.

## What you can answer today

With the data currently available (MEDSL 2018/2020/2022 for all 50 states, NC SBE 2006–2024, OpenElections for 6 states, VEST shapefiles for 4 states), you can answer:

| Question | Answer from our data |
|----------|---------------------|
| How many sheriffs ran unopposed in 2022? | 55% in North Carolina, 77% in Maine, varies by state |
| What was the closest school board race in America? | Dawson County, GA — exact tie at 25,186 to 25,186 |
| How many local races were uncontested? | 48.8% nationally (keyword-classified subset) |
| Which office type is least competitive? | Constable/Coroner at 72% uncontested |
| Which is most competitive? | City Council at 10% uncontested |
| Who has served longest on a local body in NC? | George Dunlap — Mecklenburg County Commissioner, 6 consecutive cycles (2014–2024) |
| How many unique elected offices exist in America? | At least 8,387 distinct office names in MEDSL 2022 alone; 4,995 exist in exactly one county |
| Did the same candidate run across multiple elections? | Yes — 702 NC candidates appear in 3+ election cycles (2014–2024) |

And questions you **cannot** answer yet, honestly:

| Question | Why not |
|----------|---------|
| What's the voter turnout for school board races? | Turnout data exists in less than 5% of records |
| Did this candidate switch parties? | Requires entity resolution across elections, which is functional but not yet validated at scale |
| What are the RCV round-by-round results? | Schema doesn't support ranked-choice voting yet |
| How do local election results correlate with demographics? | Census demographic join is ready (100% FIPS coverage) but not yet implemented |
| What happened in odd-year elections (2015, 2017, 2019)? | MEDSL has odd-year data on Harvard Dataverse; we haven't loaded it yet |

## The data sources

This project processes data from multiple sources. We do not redistribute their data. Here is what each provides and where to get it:

| Source | What it is | Coverage | Where to get it |
|--------|-----------|----------|-----------------|
| **[MEDSL](./sources/medsl.md)** | MIT Election Data + Science Lab precinct returns | All 50 states + DC, 2018/2020/2022 | [GitHub](https://github.com/MEDSL/2022-elections-official), [Harvard Dataverse](https://dataverse.harvard.edu/dataverse/electionscience) |
| **[NC SBE](./sources/ncsbe.md)** | North Carolina State Board of Elections | NC only, 2006–2024 (10 cycles) | [NC SBE](https://dl.ncsbe.gov/ENRS/) |
| **[OpenElections](./sources/openelections.md)** | Community-curated precinct data | ~8 states, varies | [GitHub](https://github.com/openelections) |
| **[Clarity/Scytl](./sources/clarity.md)** | Election night reporting XML | ~1,000+ jurisdictions | Per-jurisdiction URLs (unstable) |
| **[VEST](./sources/vest.md)** | Precinct results + geographic boundaries | All 50 states (shapefiles) | [Harvard Dataverse](https://dataverse.harvard.edu/dataverse/electionscience) |
| **[Census](./sources/census.md)** | FIPS code reference files | National | [Census.gov](https://www2.census.gov/geo/docs/reference/codes2020/) |
| **[FEC](./sources/fec.md)** | Federal candidate master files | National | [FEC.gov](https://www.fec.gov/data/browse-data/?tab=bulk-data) |

Each source has its own chapter documenting the exact schema, download commands, known data quality issues, and how our pipeline handles its quirks.

## Who this book is for

**If you're a journalist** and you want to answer "what happened in local elections in my area" — start with [Questions for Journalists](./problem/questions-journalists.md), then go to [Getting Started](./usage/getting-started.md) and [Recipes](./usage/recipes.md). You don't need to understand the pipeline architecture. You need the data and the queries.

**If you're a researcher** and you want a citable, reproducible, documented dataset for studying local election competitiveness, candidate career paths, or democratic participation — start with [Questions for Researchers](./problem/questions-researchers.md), then read [Reproducibility Guide](./trust/reproducibility.md) and [How to Cite This Data](./trust/citation.md). The dataset is versioned with DOIs. Every entity resolution decision is logged and auditable.

**If you're a government staffer** and you need to know what elected offices exist in your jurisdiction, how your state compares to others, or how to benchmark election administration — start with [Questions for Government Staffers](./problem/questions-government.md) and [Office Inventory Recipe](./usage/recipe-office-inventory.md).

**If you're a developer** and you want to contribute to the pipeline, add a new data source, or understand the Rust implementation — start with [Design Principles](./architecture/principles.md), then read [The Five-Layer Pipeline](./architecture/pipeline-overview.md) and [Type System Design](./rust/type-system.md). The mdbook is the spec. The Rust types are the implementation.

**If you're evaluating this architecture** for your own data pipeline project — the [Architecture](./architecture/principles.md) section describes a pattern (immutable layers, deterministic-first processing, embeddings for retrieval, LLMs for confirmation) that generalizes beyond election data. The [Hard Problems](./hard-problems/entity-resolution.md) section documents real entity resolution challenges with real data and real solutions.

## How to read this book

The book is organized in the order you'd have questions:

1. **Part I: The Problem** — Why local election data is a mess, and what questions we're trying to answer.
2. **Part II: Data Sources** — Where the data comes from, exactly what's in it, and how to download it yourself.
3. **Part III: The Hard Problems** — Name normalization, office classification, entity resolution, and cross-source reconciliation. Real examples from real data. This is the heart of the book.
4. **Part IV: Architecture** — The five-layer pipeline, the hash chain, the embedding strategy, the LLM integration. How the system is designed and why.
5. **Part V: Unified Schema** — The exact record format, field by field. What each field means, where it comes from, and which layer populates it.
6. **Part VI: Rust Implementation** — The type system, the traits, the module structure. How the architecture becomes code.
7. **Part VII: Using the Data** — Download instructions, pipeline execution, and ten ready-to-use recipes with copy-paste queries.
8. **Part VIII: Trust and Reproducibility** — How to verify the data, how to cite it, how to report errors, and what the known limitations are.

You don't have to read it in order. Every chapter is self-contained with cross-references to related sections. But if you read Part I and Part III, you'll understand why this project exists and what makes it hard. Everything else follows from that.

---

*This project is open source under MIT/Apache-2.0. The data it processes is published by its respective sources under their own licenses (generally CC-BY or public domain). We do not store or redistribute election data.*