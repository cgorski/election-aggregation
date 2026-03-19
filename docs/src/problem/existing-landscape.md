# What Exists Today and Where It Falls Short

Several organizations publish US election data. Each serves a different purpose, covers a different scope, and has different limitations. This chapter surveys the major sources and identifies the gaps that motivate this project.

## MEDSL — MIT Election Data + Science Lab

MEDSL provides the most comprehensive freely available collection of US election returns. Their datasets cover federal, state, and many local races across multiple election cycles. Data is published as flat CSV files with consistent column schemas.

**Strengths.** Wide state coverage for federal and state races. Consistent schema across years. Academic quality control. Openly licensed. Includes candidate-level vote totals with party affiliation.

**Weaknesses.** Seven states have zero local election coverage in the 2022 dataset: CA, IA, KS, NJ, PA, TN, and WI. Office name strings are not normalized — the same office appears under different names across states and years. No entity resolution across cycles (the same candidate is a new row each time). Turnout metadata is sparse. Release cadence lags elections by 12–18 months.

## ALGED — Annual Local Government Election Dataset

ALGED focuses specifically on local elections in US cities, filling a gap that most other sources ignore. It covers mayoral, city council, and some school board races.

**Strengths.** Dedicated local focus. Includes candidate demographics and incumbency status where available. Covers elections that no other academic dataset tracks.

**Weaknesses.** Limited to cities with populations above 50,000. Data collection appears to have stopped around 2021. Does not cover counties, townships, or special districts. Not currently integrated into this pipeline (planned for future work).

## OpenElections

OpenElections is a community-curated effort to collect certified election results for all 50 states. Volunteers parse state-level result files into a common CSV format and publish them on GitHub.

**Strengths.** State-level certified results for many states. Community-driven, so coverage expands over time. Raw source files are preserved alongside parsed output. Free and open.

**Weaknesses.** Coverage varies dramatically by state — some states have complete precinct-level data back to 2000, others have nothing below the county level. Schema consistency depends on the volunteer. Local races are included when the state publishes them, but there is no systematic local collection effort. Quality varies; some state files have known parsing errors that persist across releases.

## Ballotpedia

Ballotpedia maintains a wiki-style encyclopedia of US elections covering federal, state, and many local offices. Their coverage of school boards, judicial elections, and ballot measures is broader than most sources.

**Strengths.** Broad office-type coverage including judicial, school board, and special district races. Candidate biographical information. Historical coverage for some offices. Structured data behind the wiki pages.

**Weaknesses.** Bulk data access requires a commercial API license. No freely available flat-file download. Data is editorial (curated by staff, not derived from certified results). Not suitable as a primary source for vote totals, though useful for office inventories and candidate metadata.

## Associated Press (AP)

The AP provides real-time and certified election results to media organizations. Their data covers federal, state, and many local races on election night and through the canvassing period.

**Strengths.** Fast — results are available on election night. Broad geographic coverage. Includes local races in many states. High reliability for the races they cover.

**Weaknesses.** Expensive commercial license. Not available for academic or civic tech use without a contract. Historical data is not publicly archived. Coverage decisions are editorial — not all local races are included.

## Other sources

- **State election board websites** (e.g., NC SBE) publish certified results, but formats vary by state — PDF, Excel, CSV, HTML, or proprietary portals. No two states use the same schema.
- **Clarity/Scytl** election night reporting portals are used by many counties. Data is structured but ephemeral — pages are taken down or overwritten after certification.
- **VEST** (Voting and Election Science Team) provides precinct-level shapefiles matched to election returns, primarily for redistricting research. Coverage is strong for federal races but limited at the local level.
- **FEC** publishes federal candidate filings and financial data. No state or local coverage.
- **Census Bureau** provides FIPS codes and geographic hierarchies, which are essential for joining across sources but contain no election results.

## Summary

| Source | Local coverage | Schema consistency | Freely available | Current | Entity resolution |
|---|---|---|---|---|---|
| MEDSL | 43 of 50 states (2022) | High | Yes | Yes (with lag) | No |
| ALGED | Cities >50K only | Medium | Yes | No (~2021) | No |
| OpenElections | Varies by state | Low | Yes | Yes | No |
| Ballotpedia | Broad | Medium | API only | Yes | Partial |
| AP | Broad | High | No (commercial) | Yes | No |
| State portals | Varies | None (50 formats) | Usually | Yes | No |

No single source covers all local races, uses a consistent schema, resolves candidates across elections, and is freely available. That gap — between what exists and what the [four audiences](./questions.md) need — is what this project addresses.