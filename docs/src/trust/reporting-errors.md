# Reporting Errors

Election data errors are inevitable — misspelled names, transposed digits, misclassified offices. This chapter describes how to report errors, how corrections flow through the pipeline, and how they are documented.

## What counts as an error

An error is a factual discrepancy between the pipeline output and the certified source record. Examples:

- A candidate's vote total does not match the certified result.
- Two candidates are incorrectly resolved as the same person (false positive).
- A single candidate is split into two entities across sources (false negative).
- An office is classified at the wrong level (e.g., county office tagged as state).
- A contest is assigned to the wrong jurisdiction or FIPS code.

Formatting preferences (e.g., "they should use a middle name, not an initial") are not errors. The pipeline normalizes names according to documented rules; stylistic disagreements are out of scope.

## How to report

Include the following in every error report:

- **State** — two-letter abbreviation.
- **County or jurisdiction** — as specific as possible.
- **Contest** — the office name and year.
- **Candidate** — the name as it appears in the output.
- **The error** — what is wrong and what the correct value should be.
- **Source** — how you know the correct value (e.g., link to certified results PDF, county clerk confirmation).

File reports via the project's GitHub issue tracker using the `data-error` label. One error per issue. Bulk reports (e.g., "all vote totals for County X are wrong") should include a CSV attachment with the specific records.

## How corrections flow through the pipeline

Corrections are not ad hoc patches. They follow the same layered architecture as all other data.

```
Report → Review → L3 human override → L4 re-canonicalize → Changelog entry
```

1. **Report.** An error is filed with the required fields above.
2. **Review.** A maintainer verifies the error against the cited source. If the source confirms the discrepancy, the report is accepted.
3. **L3 human override.** A decision record is added to the L3 decision log with `decision_type: "human_override"`, the reporter's source citation, and the corrected value. The original machine decision is preserved — overrides do not delete history.
4. **L4 re-canonicalize.** The L4 canonical layer is regenerated from the updated L3 output. Only records affected by the override change.
5. **Changelog entry.** The correction is recorded in the [Changelog](../appendix/changelog.md) with the issue number, affected records, and the nature of the fix.

## What happens to the original data

Nothing. L0 (raw) and L1 (cleaned) records are immutable. If the error is in the source itself (e.g., the state published a wrong number that was later corrected in an amended certification), the amended source file is ingested as a new L0 record. Both the original and amended records coexist, with the L3 decision log recording which one is authoritative.

## Transparency

All override decisions are stored in the same JSONL decision log as algorithmic decisions. They are queryable, auditable, and included in pipeline replay. A consumer who disagrees with a correction can inspect the decision record, see the cited source, and file a counter-report.

Corrections do not silently change output. Every correction increments the dataset version and appears in the changelog.