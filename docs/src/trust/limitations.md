# Known Limitations

This chapter documents what the project cannot do, where the data is incomplete, and where results should be interpreted with caution. These are not future plans — they are current, known constraints.

## Coverage gaps by state

MEDSL 2022 data contains zero local election results for seven states:

| State | FIPS | Notes |
|---|---|---|
| California | 06 | State publishes results but not in MEDSL local dataset |
| Iowa | 19 | County-level results exist on state portal; not aggregated |
| Kansas | 20 | No local results in MEDSL |
| New Jersey | 34 | County clerk offices publish individually; no aggregation |
| Pennsylvania | 42 | 67 counties, each with its own reporting format |
| Tennessee | 47 | No local results in MEDSL |
| Wisconsin | 55 | State portal exists but data not present in MEDSL |

These gaps are source-dependent. If a future pipeline version integrates state portal data directly, coverage may improve. Until then, any "national" statistic derived from this dataset is actually a 43-state statistic.

## Turnout data

Turnout figures (registered voters, ballots cast) are present in fewer than 5% of records. Most sources report candidate-level vote totals but not the denominator. This means:

- Vote share (candidate votes / total ballots) cannot be computed for most contests.
- Voter participation rates at the local level are not derivable from this dataset.
- Where turnout data does exist, it is preserved as `TurnoutMetadata` contest records at L1 and carried through to L4.

Do not assume that the absence of turnout data means turnout was low. It means the source did not report it.

## Odd-year elections

Elections held in 2015, 2017, 2019, and 2021 are underrepresented. MEDSL publishes even-year datasets (2016, 2018, 2020, 2022) with strong coverage. Odd-year local elections — common for municipal and school board races — are covered only where state-specific sources (e.g., NC SBE) include them.

This creates a systematic bias: states that hold local elections in odd years appear to have fewer local races than they actually do. New Jersey (already missing from MEDSL local data) and Virginia (odd-year state legislative elections) are particularly affected.

## Entity resolution is probabilistic

The L3 matching layer uses a four-step cascade: exact match → Jaro-Winkler → embedding similarity → LLM confirmation. Only exact matches are deterministic in the strong sense. All other match methods involve thresholds:

- **Jaro-Winkler** threshold: 0.92. Names scoring below this are not matched, even if they refer to the same person.
- **Embedding cosine similarity** threshold: 0.88. Composite strings that fall below this are sent to LLM review or left unmatched.
- **LLM confirmation** is logged with a decision ID but is inherently non-deterministic across model versions. Decisions are frozen in the decision log for reproducibility, but a different model version might make different decisions.

Consequences:

- Some true matches are missed (false negatives), especially for candidates with common names in different jurisdictions.
- Some incorrect matches may exist (false positives), especially for candidates with identical names in overlapping jurisdictions (e.g., father/son with the same name).
- All non-exact match decisions are queryable by match method and score. Downstream users can apply stricter thresholds if their use case requires higher precision at the cost of lower recall.

## No ranked-choice voting (RCV) support

The schema represents first-past-the-post and plurality contests. Ranked-choice voting results — used in Alaska, Maine, New York City, and a growing number of jurisdictions — require round-by-round tabulation data that the current schema does not model.

RCV results from these jurisdictions may appear in the dataset as final-round totals (where the source reports them that way), but intermediate rounds, elimination order, and ballot transfer data are not captured.

## ALGED not integrated

The Annual Local Government Election Dataset (ALGED) covers mayoral and city council races in cities with populations above 50,000. It includes candidate demographics and incumbency data not available in other sources. This dataset is not currently integrated into the pipeline. Its coverage period ends around 2021.

Integration is planned but not scheduled. When integrated, ALGED records will enter at L0 like any other source and pass through the same cleaning, embedding, and matching layers.

## Vote mode data

Vote mode breakdowns (Election Day, absentee, early voting, provisional) are present in approximately 33% of source records. The remaining 67% report only total votes per candidate. Cross-source comparisons of vote mode data are unreliable because:

- States define vote modes differently (e.g., "absentee" vs. "mail" vs. "vote by mail").
- Some sources aggregate early voting into Election Day totals.
- Provisional ballot handling varies by state and is time-dependent (provisionals may be added days after initial reporting).

## Pipeline not validated at national scale

The pipeline has been tested against NC SBE data (2004–2022) and MEDSL data (2018–2022, 43 states). The 640-contest overlap between MEDSL and NC SBE provides a validation baseline: 90.5% exact vote match, 63% name formatting differences successfully resolved.

Full national-scale validation — running all 42 million rows through L0→L4 with cross-source reconciliation — has not been completed. Edge cases in states with unusual office structures (Louisiana's parish system, Alaska's borough system, Virginia's independent cities) may surface issues not yet encountered.

## What this means for users

If your work depends on completeness, check the [Coverage Matrix](../sources/coverage-matrix.md) for your specific state and year before drawing conclusions. If your work depends on entity resolution accuracy, filter to match methods and scores that meet your precision requirements. If your work involves RCV jurisdictions, this dataset does not capture round-level data.

These limitations are structural, not aspirational. They will change as sources are added and the pipeline matures, but they describe the current state accurately.