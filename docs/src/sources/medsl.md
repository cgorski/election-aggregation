# MEDSL — MIT Election Data + Science Lab

The MIT Election Data + Science Lab publishes precinct-level election returns for all 50 states and the District of Columbia. The data is hosted on the [Harvard Dataverse](https://dataverse.harvard.edu/dataverse/electionscience) (`electionscience` collection) and mirrored on [GitHub](https://github.com/MEDSL/2022-elections-official) for recent cycles. It is the most complete single source of US election data available without a paywall or API key.

## What MEDSL contains

MEDSL provides one CSV or tab-delimited file per state per election cycle. Each row represents one candidate in one precinct for one vote mode (election day, absentee, early voting, provisional, etc.). To obtain the total votes for a candidate in a precinct, you must sum across all rows for that candidate and precinct.

Available election cycles:

| Cycle | Location | Format | DOI |
|-------|----------|--------|-----|
| 2022 | [GitHub](https://github.com/MEDSL/2022-elections-official) | CSV, one ZIP per state | — |
| 2020 | [Harvard Dataverse](https://doi.org/10.7910/DVN/NT66Z3) | CSV/TAB, one file per state | `10.7910/DVN/NT66Z3` |
| 2018 | [Harvard Dataverse](https://doi.org/10.7910/DVN/NVQYMG) | CSV/TAB, one file per state | `10.7910/DVN/NVQYMG` |
| 2016 | [Harvard Dataverse](https://doi.org/10.7910/DVN/NH5S2I) | CSV/TAB | `10.7910/DVN/NH5S2I` |
| 2019 (odd-year) | [Harvard Dataverse](https://doi.org/10.7910/DVN/2AJUII) | CSV/TAB | `10.7910/DVN/2AJUII` |
| 2017 (odd-year) | [Harvard Dataverse](https://doi.org/10.7910/DVN/VNJAB1) | CSV/TAB | `10.7910/DVN/VNJAB1` |

We have downloaded and loaded 2018, 2020, and 2022. Together they contain approximately 36.5 million rows.

## Schema

MEDSL files have 25 columns. The delimiter is comma for most states but tab for some; auto-detection handles this.

| Column | Type | Description | Example |
|--------|------|-------------|---------|
| `precinct` | string | Precinct identifier from the source | `12-13` |
| `office` | string | Contest name, ALL CAPS | `CABARRUS COUNTY SCHOOLS BOARD OF EDUCATION` |
| `party_detailed` | string | Full party name | `NONPARTISAN` |
| `party_simplified` | string | Normalized party | `NONPARTISAN` |
| `mode` | string | Vote type for this row | `ELECTION DAY` |
| `votes` | integer | Vote count for this mode | `79` |
| `candidate` | string | Candidate name, ALL CAPS | `GREG MILLS` |
| `district` | string | District identifier or blank | `STATEWIDE`, `003`, `` |
| `dataverse` | string | Race level tag — see below | `STATE`, `SENATE`, `HOUSE`, `` |
| `stage` | string | Election stage | `GEN` |
| `special` | string | Special election flag | `FALSE` |
| `writein` | string | Write-in flag | `FALSE` |
| `date` | date | Election date | `2022-11-08` |
| `year` | integer | Election year | `2022` |
| `county_name` | string | County name, ALL CAPS | `CABARRUS` |
| `county_fips` | string | 5-digit county FIPS | `37025` |
| `jurisdiction_name` | string | Jurisdiction name | `CABARRUS` |
| `jurisdiction_fips` | string | Jurisdiction FIPS | `37025` |
| `state` | string | Full state name | `NORTH CAROLINA` |
| `state_po` | string | 2-letter postal code | `NC` |
| `state_fips` | string | 2-digit state FIPS | `37` |
| `state_cen` | string | Census state code | `56` |
| `state_ic` | string | ICPSR state code | `47` |
| `readme_check` | string | Data quality flag | `FALSE` |
| `magnitude` | integer | Number of seats in this contest | `3` |

## The `dataverse` column and local races

MEDSL tags each row with a `dataverse` value indicating which Harvard Dataverse sub-collection the race belongs to:

| Value | Meaning | Example offices |
|-------|---------|-----------------|
| `PRESIDENT` | Presidential race | President |
| `SENATE` | US Senate | US Senate |
| `HOUSE` | US House | US House District 7 |
| `STATE` | State-level offices | Governor, State Senate, Attorney General |
| *(blank)* | Everything else — **including all local races** | County Commissioner, School Board, Sheriff, Soil and Water |

Local races are identified by a **blank `dataverse` column**, not by the value `LOCAL`. This is a frequent source of confusion. In the 2022 North Carolina file, 385,260 of 684,712 rows (56%) have a blank `dataverse` value. These rows contain school board races, county commissioner races, soil and water conservation districts, district court judges, mayors, city councils, and other local offices.

In the full 2022 national dataset (12.3 million rows), approximately 5.1 million rows (41.5%) have a blank `dataverse` value.

## The `mode` column and vote totals

Each row in MEDSL represents one candidate's votes for one vote mode. A single candidate in a single precinct may have multiple rows:

```
12-13,US SENATE,LIBERTARIAN,LIBERTARIAN,ELECTION DAY,47,SHANNON W BRAY,...
12-13,US SENATE,LIBERTARIAN,LIBERTARIAN,ABSENTEE BY MAIL,5,SHANNON W BRAY,...
12-13,US SENATE,LIBERTARIAN,LIBERTARIAN,EARLY VOTING,38,SHANNON W BRAY,...
12-13,US SENATE,LIBERTARIAN,LIBERTARIAN,PROVISIONAL,0,SHANNON W BRAY,...
```

To get Shannon W. Bray's total votes in precinct 12-13, sum the `votes` column across all modes: 47 + 5 + 38 + 0 = 90.

Some states include a `TOTAL` mode row that pre-sums the other modes. Some do not. Your aggregation logic must handle both cases. If `TOTAL` rows are present, either use them directly and skip the individual mode rows, or skip `TOTAL` and sum the modes yourself. Do not double-count.

Common `mode` values: `ELECTION DAY`, `ABSENTEE BY MAIL`, `EARLY VOTING`, `ONE STOP`, `PROVISIONAL`, `TOTAL`.

## Name formatting

MEDSL candidate names are ALL CAPS with no periods after initials:

| MEDSL | Actual name |
|-------|-------------|
| `SHANNON W BRAY` | Shannon W. Bray |
| `VICTORIA P PORTER` | Victoria P. Porter |
| `MICHAEL "STEVE" HUBER` | Michael "Steve" Huber |
| `ROBERT VAN FLETCHER JR` | Robert Van Fletcher, Jr. |
| `LM "MICKEY" SIMMONS` | L.M. "Mickey" Simmons |

Nicknames appear in double quotes within the name string. Suffixes (JR, SR, III) appear without a preceding comma.

Write-in candidates are aggregated into a single row with `candidate` = `WRITEIN` and `writein` = `TRUE`.

## Non-candidate rows

Some states include metadata rows in the data that are not candidate results:

| `office` value | Meaning | Action |
|----------------|---------|--------|
| `REGISTERED VOTERS` | Voter registration count | Extract as turnout metadata, do not treat as a contest |
| `BALLOTS CAST` | Ballots cast count | Extract as turnout metadata |
| `BALLOTS CAST - TOTAL` | Same | Extract |
| `BALLOTS CAST - BLANK` | Blank ballot count | Extract |
| `STRAIGHT PARTY` | Straight-ticket party vote | Typically excluded from contest analysis |
| `OVER VOTES` | Overvote count | Extract as quality metadata |
| `UNDER VOTES` | Undervote count | Extract as quality metadata |

These rows are present in some states and absent in others. Florida OpenElections data contains 6,013 "Registered Voters" rows — 67.9% of all records classified as "other" in initial processing.

## Known coverage gaps

MEDSL 2022 contains local race data for 44 of 51 jurisdictions. Seven states have **zero rows with a blank `dataverse` column**:

| State | Likely reason |
|-------|---------------|
| California | Local results published separately by each county; not aggregated by MEDSL |
| Iowa | Local results not included in the MEDSL state file |
| Kansas | Same |
| New Jersey | Same |
| Pennsylvania | Same |
| Tennessee | Same |
| Wisconsin | Same |

This does not mean these states lack local elections. It means MEDSL's curation process did not capture them for 2022. Coverage may differ in other years.

## The `votes` column type

The `votes` column is predominantly integer, but some state files contain non-integer values. We observed:

- Floating-point values (likely vote shares erroneously placed in the votes column)
- Asterisks (`*`) indicating suppressed data
- Empty strings

Parse with `TRY_CAST` or equivalent. In our load of the full 2022 dataset, 12,782 rows had non-integer `votes` values out of 12.3 million total (0.1%).

## Download

```sh
# 2022 — All 51 files from GitHub
mkdir -p local-data/sources/medsl/2022
for state in ak al ar az ca co ct dc de fl ga hi ia id il in ks ky la \
            ma md me mi mn mo ms mt nc nd ne nh nj nm nv ny oh ok or pa \
            ri sc sd tn tx ut va vt wa wi wv wy; do
  curl -L -o "local-data/sources/medsl/2022/2022-${state}-local-precinct-general.zip" \
    "https://raw.githubusercontent.com/MEDSL/2022-elections-official/main/individual_states/2022-${state}-local-precinct-general.zip"
done

# Unzip
for f in local-data/sources/medsl/2022/*.zip; do
  unzip -o "$f" -d "${f%.zip}"
done
```

```sh
# 2020 — NC example from Harvard Dataverse (file ID 6100444)
mkdir -p local-data/sources/medsl/2020
curl -L -o local-data/sources/medsl/2020/2020-nc-precinct-general.csv \
  "https://dataverse.harvard.edu/api/access/datafile/6100444"
```

File IDs for all 51 jurisdictions in 2020 and 2018 are documented in the [download instructions](./medsl-download.md).

## Cross-source overlap

For the 2022 North Carolina general election, MEDSL and NC SBE share 640 contests where both sources report results:

- 579 (90.5%) have exactly matching vote totals
- 47 (7.3%) match within 1%
- 14 (2.2%) disagree by more than 1%
- 401 (63%) have different candidate name formatting between the two sources

This overlap is the basis for our entity resolution validation. See [Cross-Source Reconciliation](../hard-problems/cross-source.md).