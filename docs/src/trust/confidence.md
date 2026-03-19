# Confidence Levels

Every record in the pipeline carries a confidence level that reflects the trustworthiness of its source and the reliability of the processing steps applied to it. Confidence is not a score — it is a categorical label with defined semantics.

## Three levels

### High

The source is a certified government publication. Examples: NC SBE certified results, state election board portals that publish official canvass data. Records ingested from these sources enter L0 with `source.confidence = "high"`.

High-confidence sources provide vote totals that are legally authoritative. When two sources disagree, the high-confidence source is treated as ground truth.

### Medium

The source is a curated academic dataset derived from government publications. Example: MEDSL, which aggregates and reformats state-published results into a consistent schema. The data is one step removed from the original — parsed, cleaned, and sometimes corrected by the MEDSL team.

Medium-confidence sources are reliable for analysis but are not primary. In the 640 overlapping contests between MEDSL and NC SBE, 90.5% have identical vote totals. The 9.5% that differ are typically due to provisional ballot timing or reporting cutoffs.

### Low

The source is community-curated, OCR-derived, or otherwise not traceable to a single certified publication. Examples: OpenElections state files with known parsing issues, any data recovered from PDFs via OCR, or crowd-sourced contest metadata.

Low-confidence records are included in the dataset but flagged. They are useful for coverage (filling gaps where no better source exists) but should not be cited without independent verification.

## How confidence propagates

Confidence is not static. It can degrade as records pass through the pipeline, but it never improves without human intervention.

**Source confidence (L0–L1).** Set at ingestion based on the source type. Deterministic — the same source always gets the same level.

**Match confidence (L3).** Entity resolution adds a second dimension. If the match method is deterministic (exact string match or Jaro-Winkler ≥ 0.92), the source confidence is preserved. If the match required embedding similarity or LLM confirmation, the record is annotated with the match method and decision ID, but the source confidence is not downgraded — instead, a separate `match_confidence` field is added.

The combined confidence follows these rules:

| Source confidence | Match method | Overall | Notes |
|---|---|---|---|
| High | Exact | High | Best case. Certified source, deterministic match. |
| High | Jaro-Winkler | High | Algorithmic match above threshold. |
| High | Embedding | High + decision ID | Source still trusted; match is logged. |
| High | LLM | High + decision ID | Source still trusted; LLM rationale recorded. |
| Medium | Exact | Medium | Academic source, deterministic match. |
| Medium | LLM | Medium + decision ID | Both source and match carry caveats. |
| Low | Any | Low | Source uncertainty dominates. |

## LLM decision tracking

When an LLM (Claude Sonnet) is involved in entity resolution, the pipeline records:

- The decision ID (a unique hash of the prompt, response, and model version)
- The prompt sent to the model
- The model's response
- The confidence score returned by the model

This allows any LLM-assisted decision to be audited, replayed, or overridden. See [When the LLM Gets Called](../architecture/llm-when.md).

## How to cite records

When using data from this pipeline in publications, cite the **original source**, not the pipeline. The pipeline provides the information needed to construct a proper citation.

**APA format template:**

> {Source organization}. ({Year}). {Dataset title} [Data set]. Retrieved {retrieval_date} from {url}.

**Example:**

> North Carolina State Board of Elections. (2022). *Official general election results* [Data set]. Retrieved 2025-01-15 from https://www.ncsbe.gov/results-data.

Each L4 record includes the fields needed to construct this citation: `source.name`, `source.retrieval_date`, `source.url`, and `source.confidence`. A methodology link pointing to the pipeline documentation should accompany any analysis that depends on entity resolution or cross-source reconciliation.