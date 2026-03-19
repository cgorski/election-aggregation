# What We Cover, What We Don't, and Why

This page is a honest inventory of what the pipeline can and cannot do today. The status indicators mean:

- ✅ — Functional and validated
- ⚠️ — Partially implemented or not validated at scale
- ❌ — Not yet supported

## Status Table

| Capability | Status | Notes |
|------------|--------|-------|
| Precinct-level results, all 50 states | ✅ | Via MEDSL 2018/2020/2022. 36.5M rows across three cycles. |
| NC deep temporal coverage | ✅ | NC SBE 2006–2024, 10 election cycles, 2.0M+ rows. Consistent 15-column schema from 2014 onward. |
| Federal race coverage | ✅ | President, US Senate, US House present in MEDSL for all states. FEC candidate master files available for cross-referencing. |
| State-level race coverage | ✅ | Governor, state legislature, AG, SOS present in MEDSL for all states. |
| FIPS geographic enrichment | ✅ | Census reference files loaded: 3,143 counties, 31,980 places, all 50 states + DC. 100% county FIPS match rate on MEDSL data. |
| Vote mode breakdowns | ✅ | NC SBE provides Election Day / One Stop / Absentee / Provisional for every contest. MEDSL provides mode breakdowns for some states (rows split by `mode` column). |
| Local race coverage | ⚠️ | 44 of 51 MEDSL jurisdictions have local race data (blank `dataverse` column) in 2022. Seven states — CA, IA, KS, NJ, PA, TN, WI — have zero local rows. |
| Cross-source validation | ⚠️ | Validated for NC only. MEDSL and NC SBE share 640 contests in 2022: 90.5% exact vote match, 7.3% within 1%, 2.2% disagree by >1%. No systematic cross-source validation for other states. |
| Entity resolution | ⚠️ | Four-tier cascade (exact → Jaro-Winkler → embedding → LLM) is designed and prototyped. Not yet validated at scale beyond NC test cases. |
| Office classification | ⚠️ | Four-tier classifier (keyword → regex → embedding → LLM) handles 62% of 8,387 unique office names via keywords. Remaining 38% require embedding or LLM tiers. 0.56% classified as "other" in NC testing. |
| Name decomposition | ⚠️ | Parses first/middle/last/suffix/nickname from MEDSL and NC SBE formats. Handles nicknames in quotes (`"Steve"`) and parentheses (`(Steve)`). Not tested against all 50 states' formatting conventions. |
| Turnout data | ❌ | `registered_voters` and `ballots_cast` populated for <5% of records. NC SBE has "Registered Voters" pseudo-contest rows. Most MEDSL state files do not include registration counts. |
| Odd-year elections | ❌ | MEDSL publishes 2017 and 2019 on Harvard Dataverse. VEST has KY/LA/MS/VA for 2015 and 2019. None loaded into our pipeline yet. |
| Ranked-choice voting | ❌ | Schema has no fields for RCV rounds. Maine and Alaska use RCV for federal races. NYC and other cities use it for local races. No timeline for support. |
| Demographic correlation | ❌ | Census FIPS join is ready (county-level). Census demographic data (ACS) not yet integrated. The join key exists; the demographic tables do not. |
| Real-time results | ❌ | Pipeline processes certified and official results only. Not designed for election night reporting. Clarity integration (which could provide semi-live data) is not yet implemented. |
| Party switching detection | ❌ | Requires entity resolution across election cycles, which depends on L3/L4 being operational at scale. |

## Local Race Coverage Detail

The 44 states with local data in MEDSL 2022 vary in depth. Some states report thousands of local contests; others report only a handful. The seven states with zero local rows are not states without local elections — they are states where MEDSL's curation did not capture local results for that cycle.

NC SBE fills the gap for North Carolina with complete local coverage: every contest on every ballot in every precinct in all 100 counties. For other states, the gap remains.

OpenElections provides supplemental local data for FL, GA, MI, OH, PA, and TX, but coverage is inconsistent across years and granularity levels.

## What "Validated" Means

A capability marked ✅ means:

1. The data is loaded and parsed without errors.
2. The output has been spot-checked against the source.
3. Where cross-source overlap exists, the numbers have been compared.

It does not mean the data is free of errors from the source. MEDSL's `votes` column contains 12,782 non-integer values out of 12.3M rows (0.1%) in 2022. NC SBE has occasional data entry artifacts (e.g., a period after a middle name instead of a middle initial). These are source-level issues that the pipeline preserves and flags rather than silently corrects.

## What "Not Validated at Scale" Means

Entity resolution and office classification work on NC test data. We have not run them against all 42M rows across all 50 states. The algorithms are designed; the compute has not been spent. When we do run at scale, we expect to discover new edge cases — office titles we haven't seen, name formats we haven't parsed, and match ambiguities we haven't resolved.

This page will be updated as capabilities move from ⚠️ to ✅ or as new limitations are discovered.