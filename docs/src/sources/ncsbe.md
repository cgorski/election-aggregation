# NC SBE — North Carolina State Board of Elections

The North Carolina State Board of Elections publishes precinct-level results for every contest on the ballot — federal, state, and local — with vote mode breakdowns, for every election cycle back to at least 2006. It is the most complete single-state local election dataset we have found.

## What NC SBE contains

NC SBE provides one tab-delimited text file per election, delivered as a ZIP archive from an S3 bucket. Each row represents one candidate in one precinct for one contest. Vote mode totals (Election Day, early voting, absentee by mail, provisional) appear as separate columns on each row, not as separate rows. This means a single row gives you the full vote breakdown for one candidate in one precinct — unlike MEDSL, which splits each vote mode into its own row.

Coverage:

| Year | File | Rows | Notes |
|------|------|-----:|-------|
| 2024 | `results_pct_20241105.txt` | 233,511 | Presidential general |
| 2022 | `results_pct_20221108.txt` | 171,901 | Midterm general |
| 2020 | `results_pct_20201103.txt` | 257,722 | Presidential general |
| 2018 | `results_pct_20181106.txt` | 183,724 | Midterm general |
| 2016 | `results_pct_20161108.txt` | 252,827 | Presidential general |
| 2014 | `results_pct_20141104.txt` | 223,977 | Midterm general |
| 2012 | `results_pct_20121106.txt` | 208,921 | Different schema — see below |
| 2010 | `results_pct_20101102.txt` | 188,008 | Different schema |
| 2008 | `results_pct_20081104.txt` | 233,141 | Different schema |
| 2006 | `results_pct_20061107.txt` | 69,482 | Significantly different schema (9 columns) |

We have downloaded and loaded all 10 cycles. The 2014–2024 files share a stable 15-column format. Earlier files require separate parsers.

## Schema (2014–2024)

Files from 2014 onward are tab-delimited with 15 columns. There is no quoting convention; values do not contain tabs.

| Column | Type | Description | Example |
|--------|------|-------------|---------|
| `County` | string | County name, ALL CAPS | `COLUMBUS` |
| `Election Date` | string | Date as `MM/DD/YYYY` | `11/08/2022` |
| `Precinct` | string | Precinct identifier | `P17` |
| `Contest Group ID` | string | Internal contest grouping number | `7` |
| `Contest Type` | string | `S` = statewide, `C` = county/local | `C` |
| `Contest Name` | string | Full contest name, ALL CAPS | `COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02` |
| `Choice` | string | Candidate name, Title Case | `Timothy Lance` |
| `Choice Party` | string | Party abbreviation or blank | `REP`, `DEM`, ` ` |
| `Vote For` | integer | Maximum selections allowed | `1` |
| `Election Day` | integer | Election day votes | `136` |
| `One Stop` | integer | Early voting (in-person) votes | `159` |
| `Absentee by Mail` | integer | Mail absentee votes | `7` |
| `Provisional` | integer | Provisional ballot votes | `1` |
| `Total Votes` | integer | Sum of all vote modes | `303` |
| `Real Precinct` | string | `Y` = physical precinct, `N` = aggregation group | `Y` |

## The `Contest Type` column

The `Contest Type` field distinguishes local from statewide races:

- `C` — county/local contests: school board, county commissioner, city council, soil and water, local judicial races, bond referendums
- `S` — statewide contests: US Senate, US House, Governor, state legislature, statewide judicial races

For local election analysis, filter to `Contest Type = 'C'`. In the 2022 file, this yields 919 distinct contests across 100 counties.

## Vote mode columns

NC SBE is the only source in our corpus that provides vote mode breakdowns as columns for every contest, including local races. The four modes are:

| Column | Meaning |
|--------|---------|
| `Election Day` | Votes cast in person on election day |
| `One Stop` | Early in-person voting (North Carolina's term for early voting) |
| `Absentee by Mail` | Absentee ballots returned by mail |
| `Provisional` | Provisional ballots accepted during canvass |

`Total Votes` is the sum of the four mode columns. We have verified this holds across all rows in the 2014–2024 files.

The vote mode data enables analysis that most sources cannot support: comparing early voting patterns to election day patterns at the precinct level for local races. Three of our nine data sources provide any vote mode breakdown at all (NC SBE, Clarity, and MEDSL for some states). NC SBE is the only one that provides it consistently for all contests.

## Non-contest rows

NC SBE data includes rows that are not candidate results. These appear as entries in the `Choice` column within contests that are not real races:

| `Contest Name` pattern | `Choice` value | What it is |
|------------------------|----------------|------------|
| Contains "Registered Voters" | (varies) | Voter registration count for the precinct |
| Any contest | `Write-In (Miscellaneous)` | Aggregated write-in votes |
| Any contest | `Over Votes` | Overvote count |
| Any contest | `Under Votes` | Undervote count |

The "Registered Voters" rows deserve special attention. They appear as a contest named "Registered Voters" with a single `Choice` entry where `Total Votes` contains the number of registered voters in that precinct. This is turnout metadata, not a contest result.

In our prototype pipeline, we extract the registered voter count from these rows into a turnout object, then exclude the row from contest analysis. This is how we backfill the `turnout.registered_voters` field that is otherwise unpopulated for most sources.

Write-in rows with the suffix `(Write-In)` in the candidate name (e.g., `Ronnie Strickland (Write-In)`) are distinct from the aggregated `Write-In (Miscellaneous)` row. The named write-in rows report votes for a specific write-in candidate. The `(Miscellaneous)` row reports the total for all unnamed write-ins.

## The `Real Precinct` column

`Real Precinct = Y` indicates a physical voting precinct with a defined geographic boundary. `Real Precinct = N` indicates an aggregation group — typically used for absentee-only tallies or provisional ballot pools that cannot be assigned to a specific precinct.

For geographic analysis (mapping, precinct-level comparison), filter to `Real Precinct = 'Y'`. For total vote counts, include both.

## Candidate name formatting

NC SBE candidate names are Title Case with periods after initials and commas before suffixes:

| NC SBE | Components |
|--------|------------|
| `Timothy Lance` | first=Timothy, last=Lance |
| `Shannon W. Bray` | first=Shannon, middle=W, last=Bray |
| `Robert Van Fletcher, Jr.` | first=Robert, middle=Van, last=Fletcher, suffix=Jr. |
| `Michael (Steve) Huber` | first=Michael, nickname=Steve, last=Huber |
| `William Irvin. Enzor III` | first=William, middle=Irvin, last=Enzor, suffix=III |
| `Patricia (Pat) Cotham` | first=Patricia, nickname=Pat, last=Cotham |

Nicknames appear in parentheses. This differs from MEDSL, which uses double quotes. The period after "Irvin." in "William Irvin. Enzor III" appears to be a data entry artifact — the period belongs after the middle initial, not after the full middle name. These inconsistencies are present in the source data and must be handled during name decomposition at L1.

## Schema changes across years

The 2014–2024 files share the 15-column schema documented above. Earlier files differ:

**2008–2012**: The schema has 14–15 columns but with different names and ordering. `Contest Type` is the third column (not the fifth). Fields are comma-delimited with quote wrapping. The `district` column was added later. Vote mode columns use slightly different names in some years.

**2006**: Significantly different. Only 9 columns: `county`, `election_dt`, `precinct_abbrv`, `precinct`, `contest_name`, `name_on_ballot`, `party_cd`, `ballot_count`, `FTP_Date`. No vote mode breakdown. No `Contest Type` field. All column names are lowercase with underscores.

We currently parse 2014–2024 with one parser and treat 2006–2012 as a separate parser target. The 2008–2012 files contain local races (they have `Contest Type = C`) but require different column mapping. The 2006 file requires more investigation to determine whether it includes local races.

## Why NC SBE matters

NC SBE is not the largest dataset in our corpus (MEDSL has far more rows). Its value is in three properties that no other source provides simultaneously:

1. **Complete local coverage.** Every contest on every ballot in every precinct in every county — school board, soil and water, county commissioner, municipal, judicial, and bond referendums. MEDSL has gaps in local race coverage for some states. NC SBE has none for North Carolina.

2. **Vote mode breakdowns for local races.** The four-column mode breakdown (Election Day, One Stop, Absentee, Provisional) is present for every contest, including hyperlocal races like "Whiteville City Schools Board of Education District 01."

3. **Ten-year temporal depth.** Six clean election cycles (2014–2024) with a consistent schema. This enables career tracking, competitiveness trend analysis, and temporal chain construction across a decade of local elections. Combined with the 2008–2012 files (once parsed), the coverage extends to nearly 20 years.

The combination of these three properties makes NC SBE the primary validation dataset for the pipeline. When we test cross-source entity resolution, we compare MEDSL NC against NC SBE NC — 640 overlapping contests with 90.5% exact vote total agreement and 63% candidate name formatting differences. When we test temporal chains, we track candidates across NC SBE's six-cycle span.

## Download

The URL pattern is:

```
https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/{YYYY_MM_DD}/results_pct_{YYYYMMDD}.zip
```

```sh
mkdir -p local-data/sources/ncsbe/{2014,2016,2018,2020,2022,2024}

# 2024
curl -L -o local-data/sources/ncsbe/2024/results_pct_20241105.zip \
  "https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2024_11_05/results_pct_20241105.zip"

# 2022
curl -L -o local-data/sources/ncsbe/2022/results_pct_20221108.zip \
  "https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2022_11_08/results_pct_20221108.zip"

# 2020
curl -L -o local-data/sources/ncsbe/2020/results_pct_20201103.zip \
  "https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2020_11_03/results_pct_20201103.zip"

# 2018
curl -L -o local-data/sources/ncsbe/2018/results_pct_20181106.zip \
  "https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2018_11_06/results_pct_20181106.zip"

# 2016
curl -L -o local-data/sources/ncsbe/2016/results_pct_20161108.zip \
  "https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2016_11_08/results_pct_20161108.zip"

# 2014
curl -L -o local-data/sources/ncsbe/2014/results_pct_20141104.zip \
  "https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2014_11_04/results_pct_20141104.zip"

# Unzip all
for d in local-data/sources/ncsbe/*/; do
  cd "$d" && unzip -o *.zip && cd -
done
```

Older cycles (2008–2012) follow the same URL pattern. The 2006 file uses a different path structure. The full election calendar is at [ncsbe.gov/results-data](https://www.ncsbe.gov/results-data).