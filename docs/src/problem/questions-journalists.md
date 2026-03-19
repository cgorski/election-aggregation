# For Journalists

Local election data is where accountability stories live — and where data is hardest to find. These are the questions journalists ask, with real answers drawn from the dataset.

## Closest races

- **Who won the closest race in America?** Dawson County, GA had a tied contest at 25,186 total votes cast — decided by recount procedures, not by a single voter's margin.
- **How many exact ties exist?** 19 exact ties have been identified across available data. Each is flagged with the specific contest, county, and vote totals. See [Closest Races in America](../usage/recipe-closest-races.md).
- **Which school board races were decided by single digits?** Madison County, IN had a school board race decided by 1 vote. These contests are queryable by margin across all office types.

## Unopposed races

- **How many sheriffs ran unopposed?** In North Carolina, 55% of sheriff races were uncontested. In Maine, 77%. National figures depend on source coverage — seven states lack local data entirely.
- **What's the overall uncontested rate?** 48.8% of local races in available data have a single candidate. This figure spans all office types and all states with coverage.
- **Which offices are most likely to be uncontested?** Constable races are uncontested 72% of the time. City council races: 10%. The rate varies by office type and state. See [Uncontested Race Rate by State](../usage/recipe-uncontested.md).

## Accountability angles

- **Who keeps winning without opposition?** Candidate entity resolution across election cycles identifies incumbents who have never faced an opponent. See [Career Tracking Across Elections](../usage/recipe-career-tracking.md).
- **Which counties have the most uncontested offices?** County-level aggregation is possible wherever FIPS codes are present in the source data. See [Sheriff Accountability](../usage/recipe-sheriffs.md).
- **Are there races where write-in candidates are the only opposition?** Write-in totals are preserved where the source reports them. In some jurisdictions, write-in votes account for the only opposition in over a third of contests.

## Verification

- **Can I verify a specific result?** Every record traces back to a named source (e.g., NC SBE certified results, MEDSL). The pipeline preserves source file hashes and original field values. See [Verify a Specific Result](../usage/recipe-verify-result.md).
- **How do I cite this data?** You cite the original source, not this project. The project provides the source name, retrieval date, and confidence level for each record. See [Confidence Levels](../trust/confidence.md).

## What you cannot get here (yet)

- Turnout data is present in fewer than 5% of records.
- Seven states (CA, IA, KS, NJ, PA, TN, WI) have zero local coverage in MEDSL 2022.
- Odd-year elections (2015, 2017, 2019, 2021) are underrepresented.

These gaps are documented in [Known Limitations](../trust/limitations.md). If you are reporting on a state with limited coverage, check the [Coverage Matrix](../sources/coverage-matrix.md) first.