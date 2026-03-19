# Future Sources

This chapter documents data sources that have been identified as valuable but are not yet integrated into the pipeline. Each is blocked by a specific access, cost, or engineering constraint.

## ALGED — Annual Local Government Election Data

The Annual Local Government Election Data project, hosted on the Open Science Framework (OSF), covers municipal elections in 1,747 cities with populations over 25,000. Records include candidate demographics (race, gender), incumbency status, and election outcomes — fields that no other source in our corpus provides.

**Coverage**: Municipal elections from 2000–2020. Cities only (no counties, no school districts). Focuses on mayoral and city council races.

**Format**: CSV files organized by city population tier.

**Status**: Blocked. The OSF repository requires an approved access request. We submitted a request in early 2025 and have not received a response. The underlying data appears to be derived from individual city clerk records, manually curated by the research team.

**Value if integrated**: ALGED would fill the demographic gap entirely. No other source provides candidate race or gender. It would also provide an independent validation source for municipal races in the 1,747 covered cities.

## Ballotpedia

Ballotpedia maintains the most comprehensive database of US local elections, covering school boards, city councils, county commissions, judges, ballot measures, and special districts across all 50 states. Their coverage extends to races that no other source tracks — mosquito abatement districts, water boards, and transit authorities.

**Coverage**: All 50 states, all levels of government, ongoing since approximately 2007.

**Format**: Structured database accessible via a paid API. Some data is available on the public website but is not bulk-downloadable.

**Status**: Blocked by cost. The API requires a commercial license. Pricing is not publicly listed but is reported to be in the five-figure annual range. We have not pursued a license.

**Value if integrated**: Ballotpedia would be the single largest improvement to local race coverage. It would fill the 7-state local race gap in MEDSL 2022 and provide office-level metadata (term length, salary, appointing authority) that no source currently offers.

## AP Elections API

The Associated Press Elections API provides real-time and certified results for federal and state races, with some local coverage in larger jurisdictions. It is the standard data feed used by newsrooms on election night.

**Coverage**: Federal and statewide races nationwide. County-level results for most races. Precinct-level for some states. Local race coverage varies.

**Format**: JSON API with WebSocket push for live updates.

**Status**: Blocked by cost. The AP API is a commercial product priced for newsroom budgets. It is not available for academic or open-source use without a contract. The real-time capability is irrelevant to our pipeline (we process certified results, not live feeds), but the certified result snapshots would be a valuable validation source.

**Value if integrated**: AP results would serve as a third independent source for federal and statewide races, enabling three-way cross-source validation alongside MEDSL and state portals. AP's candidate identifiers are stable across cycles, which would simplify temporal chaining for federal candidates.

## Additional State Portals

Six states with significant populations publish precinct-level results through their own election portals in structured formats. These would complement MEDSL by providing certified results directly from the state authority.

**Florida**: The Division of Elections publishes precinct-level results at `results.elections.myflorida.com`. CSV format. All counties, all contests. Would overlap with both MEDSL and OpenElections FL, enabling three-source validation for one state.

**Georgia**: The Secretary of State publishes results at `results.enr.clarityelections.com/Georgia/` (Clarity-based) and via a separate certified results portal. XML and CSV. Would provide a second source for GA alongside MEDSL.

**Texas**: The Secretary of State publishes county-level results (not precinct-level) at `elections.sos.state.tx.us`. Precinct-level results are published by individual counties. A full TX integration would require crawling 254 county websites or using the Clarity instances that many TX counties operate.

**Ohio**: The Secretary of State publishes precinct-level results at `www.ohiosos.gov/elections/election-results-and-data/`. CSV format. Covers all contests including local races.

**Pennsylvania**: The Department of State publishes results at `electionreturns.pa.gov`. JSON API available. Covers all contests. Would fill one of the 7 states with zero local data in MEDSL 2022.

**Michigan**: The Secretary of State publishes precinct-level results at `miboecfr.nictusa.com/cgi-bin/cfr/`. Older web interface with downloadable files. Covers all contests.

**Status**: Not blocked by access — all six portals are public. Blocked by engineering time. Each state portal has its own format, URL structure, and quirks. We estimate 1–2 weeks of parser development per state. These are the highest-priority engineering tasks after odd-year MEDSL loading.