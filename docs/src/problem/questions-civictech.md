# For Civic Tech Developers

Civic technology projects depend on structured, reliable election data. Most fail not because of engineering limitations but because the underlying data is fragmented, inconsistently formatted, and difficult to resolve across sources. These are the questions developers ask when building on local election data.

## Ballot lookup tools

- **Can I build a "what's on my ballot" tool?** Yes, but it requires mapping voter addresses to jurisdictions (via OCD-IDs or FIPS codes) and then mapping jurisdictions to offices. The dataset contains 8,387 unique office name strings — many of which refer to the same office across sources. The L4 canonical layer resolves these to deduplicated office records with jurisdiction identifiers.
- **How do I map an address to its contests?** You need an OCD-ID → office mapping. OCD-IDs (Open Civic Data Identifiers) are present where source data includes them or where FIPS codes allow deterministic derivation. Coverage is not universal. See the [Schema Overview](../schema/overview.md) for the `jurisdiction.ocd_id` field.
- **What format is the data in?** Every pipeline layer outputs JSONL (newline-delimited JSON). One record per line, one file per source-year-state. No database required — parse with `jq`, Python, DuckDB, or any JSON-capable tool.

## Candidate lookup and entity resolution

- **Can I build a candidate lookup API?** The L4 layer provides entity-resolved candidate records with canonical names, office history, and source attribution. A candidate who appears as "Bill Smith" in one source and "William R. Smith Jr." in another is resolved to a single entity with both name variants preserved.
- **How reliable is entity resolution?** It depends on the match method. Exact matches and high-confidence Jaro-Winkler matches (≥0.92) are deterministic. Embedding-based and LLM-confirmed matches carry a decision ID that traces back to the specific match rationale. See [The Cascade](../hard-problems/entity-cascade.md).
- **Can I track candidates across election cycles?** Yes. Entity resolution operates across years. George Dunlap in Mecklenburg County, NC appears across 6 election cycles with consistent entity IDs. See [Career Tracking](../usage/recipe-career-tracking.md).

## Election history and widgets

- **Can I build an election history widget for a jurisdiction?** The data supports historical queries by jurisdiction, office, and candidate. Time series depend on source coverage — MEDSL covers 2018–2022 for most states; NC SBE covers 2004–present for North Carolina.
- **What about ballot measures?** Ballot measures are a distinct contest kind (`BallotMeasure`) in the schema. Choices are normalized to `for`/`against`/`yes`/`no` at L1.

## Data interchange

- **Why JSONL and not a REST API?** JSONL is the data interchange format at every layer. It is self-describing, streamable, and requires no server infrastructure. Downstream applications can ingest it directly or load it into any datastore.
- **Can I join this data with other civic datasets?** Yes. Records include FIPS codes, OCD-IDs (where available), and state abbreviations. These are standard join keys for Census data, geographic boundaries, and other civic datasets.
- **Is the schema stable?** The schema is versioned. Each JSONL record includes a `schema_version` field. Breaking changes increment the major version. See [Schema Overview](../schema/overview.md).

## What to watch out for

- The project does not host a live API or data download. It documents sources and provides pipeline tools to process them. You run the pipeline yourself.
- Coverage gaps exist. Seven states lack local data in MEDSL 2022. Odd-year elections are underrepresented. Check the [Coverage Matrix](../sources/coverage-matrix.md) before building features that assume national coverage.
- Entity resolution is probabilistic for non-exact matches. If your application requires certainty, filter to records with `match_method: "exact"` or `match_method: "jaro_winkler"`.