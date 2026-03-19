# For Researchers

Local election data presents structural challenges for quantitative research: inconsistent office names, no universal candidate identifiers, and source-dependent coverage gaps. These are the questions researchers ask, with real answers and methodology notes.

## Competitiveness and contestation

- **What's the uncontested rate by office type?** Constable: 72%. Soil and water conservation district: 58%. County commissioner: 34%. City council: 10%. These rates are computed from L4 canonical records where `candidate_count = 1` for a given contest.
- **How does competitiveness vary across states?** Minnesota reports 89.3% of local races as contested. Florida reports 0% in available MEDSL local data — a coverage artifact, not a political finding. Interpret cross-state comparisons with the [Coverage Matrix](../sources/coverage-matrix.md).
- **What is the national uncontested rate?** 48.8% across all available local races. This figure is coverage-weighted: states with more reported contests contribute proportionally more. It is not a population-weighted estimate.

## Candidate career tracking

- **Can I track candidates across election cycles?** Entity resolution at L3 links candidate records across years and sources. George Dunlap (Mecklenburg County, NC) appears in 6 election cycles under consistent entity IDs. See [Career Tracking Across Elections](../usage/recipe-career-tracking.md).
- **What identifier links candidates across sources?** The L4 `canonical_candidate_id` is a deterministic hash of resolved name components, jurisdiction, and office. It is stable across pipeline runs given the same L3 decisions.
- **How reliable is cross-cycle linking?** Exact name matches are deterministic. Fuzzy matches (Jaro-Winkler ≥ 0.92) and embedding matches (cosine ≥ 0.88) are logged with scores. LLM-assisted matches include the decision ID. All match metadata is queryable.

## Cross-source validation

- **How consistent are sources that cover the same contests?** In 640 overlapping contests between MEDSL and NC SBE, 90.5% have identical vote totals. The remaining 9.5% differ by small amounts, typically due to provisional ballot timing or reporting cutoff dates.
- **How do candidate names differ across sources?** In the same 640 overlapping contests, 63% have name formatting differences (e.g., "SMITH, JOHN A" vs. "John A. Smith"). These are resolved at L1 (parsing) and confirmed at L3 (entity resolution).

## Office taxonomy

- **How many distinct offices exist?** 8,387 unique office name strings before normalization. After L2 classification (keyword, regex, embedding, LLM), these resolve to a smaller set of canonical office types. See [Office Classification Reference](../appendix/office-classification.md).
- **What office types exist at the sub-county level?** Constable, justice of the peace, soil and water conservation district supervisor, school board trustee, municipal utility district director, and hundreds of jurisdiction-specific titles.

## Reproducibility

All findings above are reproducible from the pipeline output:

- **L0 → L2** layers are fully deterministic. Given the same source files and pipeline version, output is byte-identical.
- **L3** decisions are logged in a decision log (JSONL). Replaying the log against the same L2 input reproduces L3 exactly, even when LLM calls were involved.
- **L4** is deterministic given L3 output.
- **Versioned JSONL** files at every layer serve as the unit of reproducibility. Each file includes a manifest with source hashes, pipeline version, and timestamp.

To reproduce a specific finding, check out the tagged pipeline version, supply the same L0 inputs, and run the pipeline. The decision log ensures that even probabilistic steps (embedding similarity, LLM confirmation) produce identical output on replay.

## Data format for analysis

Pipeline output is JSONL — one JSON object per line. This is directly loadable into pandas, R (`jsonlite`), DuckDB, or any tool that reads newline-delimited JSON. No proprietary formats or database dependencies are required.