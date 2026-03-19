# Source Overview

This project ingests election data from seven sources. None are complete on their own. Each fills a different gap — geographic breadth, temporal depth, local race coverage, geographic boundaries, or reference identifiers. The pipeline merges them into a unified schema; this chapter documents what each provides and where they overlap.

## Source Summary

| Source | What It Provides | Coverage | Format | Access Method |
|--------|-----------------|----------|--------|---------------|
| [MEDSL](./medsl.md) | Precinct-level returns for federal, state, and some local races | 50 states + DC; 2018, 2020, 2022 (~36.5M rows) | CSV/TSV, one file per state per cycle | Harvard Dataverse download, GitHub mirror |
| [NC SBE](./ncsbe.md) | Precinct-level returns for every contest on the ballot, with vote mode breakdowns | NC only; 2006–2024 (10 cycles, ~2M rows) | Tab-delimited TXT in ZIP archives | S3 bucket direct download |
| [OpenElections](./openelections.md) | Community-curated precinct-level CSV files | ~8 states with 2022 data (FL, GA, MI, OH, PA, TX, others); coverage varies | CSV, schema varies by state | Git clone per state repo on GitHub |
| [Clarity/Scytl](./clarity.md) | Election night reporting with precinct-level XML results | ~1,000+ jurisdictions nationwide | Structured XML in ZIP files | Per-jurisdiction URLs (unstable across cycles) |
| [VEST](./vest.md) | Precinct boundaries (shapefiles) with vote counts as attributes | 50 states; odd-year elections for KY/LA/MS/VA (2015, 2019) | Shapefile (.shp/.dbf/.shx/.prj) | Harvard Dataverse download |
| [Census](./census.md) | FIPS reference codes for states (50+DC), counties (3,143), and places (31,980) | National, 2020 vintage | Pipe-delimited text files | census.gov direct download |
| [FEC](./fec.md) | Federal candidate master records with stable `CAND_ID` identifiers | All registered federal candidates; 2020 and 2022 loaded | Pipe-delimited TXT (`cn.txt`) in ZIP | fec.gov bulk download |

## What Each Source Contributes to the Pipeline

**MEDSL** is the backbone. It covers all 50 states at precinct granularity for three recent even-year cycles. Approximately 41.5% of rows in the 2022 dataset have a blank `dataverse` column, indicating local races. Seven states have zero local race rows — see [Coverage Matrix](./coverage-matrix.md).

**NC SBE** provides the deepest single-state coverage: every contest on every ballot in every precinct across 10 election cycles. It is the only source that provides vote mode breakdowns (Election Day, early, absentee, provisional) for local races. It serves as the primary validation dataset for cross-source entity resolution.

**OpenElections** fills state-level gaps where MEDSL coverage is incomplete or where an alternative source view aids cross-validation. Schema varies by state, requiring per-state parser logic.

**Clarity** has the highest value for hyperlocal races (school board, city council, judicial) because it captures results directly from county ENR systems. Not yet integrated in our pipeline. URL instability is the primary obstacle.

**VEST** provides the only precinct boundary geometries in the corpus, enabling geographic analysis. It also covers odd-year elections (2015, 2019) for states with off-cycle gubernatorial races — data that MEDSL's loaded cycles do not include.

**Census** provides the authoritative FIPS code-to-name mappings used at L1 for geographic enrichment and cross-source geographic joins.

**FEC** provides stable candidate identifiers (`CAND_ID`) for federal candidates, used at L3 as reference anchors during entity resolution.

## Cross-Source Overlap

Two source pairs have been compared quantitatively.

### MEDSL + NC SBE (North Carolina, 2022 General)

Both sources report precinct-level results for the same 640 contests in North Carolina's 2022 general election. Comparison results:

| Metric | Value |
|--------|-------|
| Contests with exact vote total match | 579 (90.5%) |
| Contests matching within 1% | 47 (7.3%) |
| Contests disagreeing by >1% | 14 (2.2%) |
| Contests with different candidate name formatting | 401 (63%) |

The 63% name formatting difference rate is the reason entity resolution exists. MEDSL reports `SHANNON W BRAY` (all caps, no period). NC SBE reports `Shannon W. Bray` (title case, period after initial). Same person, different string. This overlap is the primary test bed for the matching pipeline — see [Cross-Source Reconciliation](../hard-problems/cross-source.md).

### MEDSL + OpenElections (Florida, 2022 General)

Florida OpenElections data contains 6,013 "Registered Voters" rows (67.9% of non-candidate records), which are turnout metadata rows mixed into the results file. This overlap revealed the non-candidate row problem documented in [Non-Candidate Records](../hard-problems/non-candidates.md).

## Source Priority Ranking

When multiple sources report results for the same contest, the pipeline applies a priority order to select the authoritative record:

| Priority | Source Type | Rationale | Examples |
|----------|-----------|-----------|----------|
| 1 | Certified state data | Published by the official election authority; legally authoritative | NC SBE |
| 2 | Academic curated | Cleaned and standardized by researchers with documented methodology | MEDSL, VEST |
| 3 | Community curated | Volunteer-driven; quality varies by state and contributor | OpenElections |
| 4 | Election night reporting | Often preliminary, not certified; URLs are unstable | Clarity |
| 5 | Reference only | Not election results; used for enrichment and cross-referencing | Census, FEC |

Priority 1 sources are preferred when available. In practice, NC SBE is the only certified state source currently loaded. For the remaining 49 states, MEDSL (priority 2) is the primary source. Lower-priority sources are retained in the record's provenance for cross-validation, not discarded.

The priority ranking affects two pipeline decisions: which record becomes the canonical version at L4, and which confidence level is assigned. A record confirmed by two independent sources (e.g., MEDSL + NC SBE with matching vote totals) receives `High` confidence. A record from a single source receives `Medium` or `Low` depending on the source tier.