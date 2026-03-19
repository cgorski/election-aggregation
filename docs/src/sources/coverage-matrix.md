# Coverage Matrix

This chapter maps which sources cover which states and years. Use it to determine whether a specific state/year/level combination is available before querying.

## MEDSL — 50 States, 3 Cycles

MEDSL provides precinct-level results for all 50 states plus DC across three even-year general election cycles. Each cycle is one CSV per state.

| Cycle | States | Approximate rows | Local race coverage |
|-------|--------|------------------:|---------------------|
| 2018  | 50 + DC | ~11.0M | Varies by state |
| 2020  | 50 + DC | ~13.2M | Varies by state |
| 2022  | 50 + DC | ~12.3M | 44 of 51 jurisdictions |

**Seven states with zero local data in MEDSL 2022.** These states have no rows with a blank `dataverse` column, meaning no local races were captured:

| State | FIPS |
|-------|------|
| California | 06 |
| Iowa | 19 |
| Kansas | 20 |
| New Jersey | 34 |
| Pennsylvania | 42 |
| Tennessee | 47 |
| Wisconsin | 55 |

Local elections occur in all seven states. MEDSL's curation process did not capture them for 2022. Coverage may differ in 2018 and 2020.

**Odd-year data on Dataverse but not yet loaded.** MEDSL publishes odd-year election data on Harvard Dataverse:

| Cycle | DOI | Status |
|-------|-----|--------|
| 2015  | — | Not loaded |
| 2017  | `10.7910/DVN/VNJAB1` | Not loaded |
| 2019  | `10.7910/DVN/2AJUII` | Not loaded |
| 2021  | — | Not loaded |

Odd-year elections cover gubernatorial races in VA, NJ, KY, LA, MS and municipal elections in many states. Loading these would fill a significant gap.

## NC SBE — 1 State, 10 Cycles

NC SBE covers North Carolina exclusively, with precinct-level results for every contest on the ballot.

| Year | Election | Rows | Schema |
|------|----------|-----:|--------|
| 2024 | General | 233,511 | 15-column |
| 2022 | General | 171,901 | 15-column |
| 2020 | General | 257,722 | 15-column |
| 2018 | General | 183,724 | 15-column |
| 2016 | General | 252,827 | 15-column |
| 2014 | General | 223,977 | 15-column |
| 2012 | General | 208,921 | 14–15 column (different layout) |
| 2010 | General | 188,008 | 14–15 column (different layout) |
| 2008 | General | 233,141 | 14–15 column (different layout) |
| 2006 | General | 69,482 | 9-column (significantly different) |

All 10 cycles are downloaded. The 2014–2024 files share a stable schema and a single parser. The 2008–2012 files require a separate parser. The 2006 file requires a third.

## OpenElections — ~8 States, Variable Coverage

OpenElections is community-curated. Coverage depends on volunteer effort per state. The following states have 2022 precinct-level general election data:

| State | 2022 precinct data | Earlier years |
|-------|-------------------|---------------|
| Florida | ✅ | 2000–2020 |
| Georgia | ✅ | 2004–2020 |
| Michigan | ✅ | 2000–2020 |
| Ohio | ✅ | 2000–2020 |
| Pennsylvania | ✅ | 2000–2020 |
| Texas | ✅ | 2000–2020 |
| North Carolina | ✅ | 2008–2020 |
| Arizona | Partial | 2004–2020 |

Coverage for other states exists at county level or for federal races only. Check each state's GitHub repository (`openelections-data-{state}`) for current status.

## VEST — Shapefiles with Vote Counts

VEST publishes precinct-level shapefiles for all 50 states. We have loaded a subset for odd-year coverage:

| State | Year | Election type | Loaded |
|-------|------|---------------|--------|
| Kentucky | 2019 | General (Governor) | ✅ |
| Louisiana | 2019 | General (Governor) | ✅ |
| Mississippi | 2019 | General (Governor) | ✅ |
| Virginia | 2019 | General (state legislature) | ✅ |
| Kentucky | 2015 | General (Governor) | ✅ |
| Louisiana | 2015 | General (Governor) | ✅ |
| Mississippi | 2015 | General (Governor) | ✅ |
| Virginia | 2015 | General (state legislature) | ✅ |

VEST covers state-level races only (president, governor, US Senate, US House, state legislature). No local races.

## Census and FEC — Reference Data

These are not election results. They provide reference identifiers used during pipeline enrichment.

| Source | Scope | Years | Records |
|--------|-------|-------|--------:|
| Census county FIPS | National | 2020 | 3,143 |
| Census place FIPS | National | 2020 | 31,980 |
| Census state FIPS | National | 2020 | 56 |
| FEC candidate master | Federal candidates | 2020 | ~6,800 |
| FEC candidate master | Federal candidates | 2022 | ~6,600 |

## Clarity/Scytl — Not Yet Integrated

Clarity ENR sites cover 1,000+ jurisdictions but are not yet in the pipeline. URLs are unstable across election cycles, making systematic acquisition difficult. See [Clarity/Scytl ENR](./clarity.md).

## Combined Coverage Summary

| Dimension | Current status |
|-----------|---------------|
| States with any data | 50 + DC |
| Even-year general elections | 2018, 2020, 2022 |
| Odd-year elections | KY/LA/MS/VA 2015, 2019 (VEST only, state-level) |
| Deep single-state coverage | NC, 2006–2024 (10 cycles) |
| Total rows across all sources | ~42M |
| Local race coverage | 44 of 51 jurisdictions (MEDSL 2022) + NC (NC SBE) |
| Vote mode breakdowns | NC SBE (all contests), MEDSL (some states), Clarity (when integrated) |
| Turnout data | <5% of records populated |

## Gap Analysis

**Temporal gaps.** No odd-year municipal election results are loaded. Cities like New York, Los Angeles, Houston, Philadelphia, and San Antonio hold elections in odd years. MEDSL publishes 2017 and 2019 data on Dataverse. Loading these would add coverage for the largest US cities.

**State-level local gaps.** Seven states have zero local race data in MEDSL 2022. OpenElections partially fills this for Pennsylvania. The remaining six (CA, IA, KS, NJ, TN, WI) require either Clarity integration or direct state portal downloads.

**Primary elections.** All loaded data is general election only. MEDSL tags primary results with `stage = PRI` but we have not loaded primary-specific files. NC SBE publishes primary results as separate files.

**Runoff elections.** Georgia, Louisiana, Texas, and other states hold runoff elections. These are partially captured in MEDSL (`stage = RUN`) but not systematically loaded.