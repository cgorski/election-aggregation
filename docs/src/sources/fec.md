# FEC — Federal Election Commission Candidate Master Files

The FEC publishes bulk data files for every registered federal candidate: President, US Senate, and US House. These files provide stable candidate identifiers (`CAND_ID`) that persist across election cycles, making them a reference source for cross-linking federal candidates across MEDSL, NC SBE, and OpenElections data.

## What FEC provides

The candidate master file (`cn.txt`) contains one row per candidate per election cycle. It covers all candidates who have filed with the FEC, including those who lost primaries or never appeared on a general election ballot.

Available cycles: 1980–present. We have downloaded 2020 and 2022.

## Download

Bulk data is at [fec.gov/data/browse-data](https://www.fec.gov/data/browse-data/?tab=bulk-data).

```sh
mkdir -p local-data/sources/fec/{2020,2022}

# 2022
curl -L -o local-data/sources/fec/2022/cn.zip \
  "https://www.fec.gov/files/bulk-downloads/2022/cn.zip"
unzip -o local-data/sources/fec/2022/cn.zip -d local-data/sources/fec/2022/

# 2020
curl -L -o local-data/sources/fec/2020/cn.zip \
  "https://www.fec.gov/files/bulk-downloads/2020/cn.zip"
unzip -o local-data/sources/fec/2020/cn.zip -d local-data/sources/fec/2020/
```

## Schema

The file `cn.txt` is pipe-delimited (`|`) with **15 columns** and no header row.

| # | Column | Description | Example |
|---|--------|-------------|---------|
| 1 | `CAND_ID` | Stable candidate identifier | `H0NC09072` |
| 2 | `CAND_NAME` | Name in LAST, FIRST MIDDLE format | `BRAY, SHANNON W` |
| 3 | `CAND_PTY_AFFILIATION` | Party code | `LIB` |
| 4 | `CAND_ELECTION_YR` | Election year | `2022` |
| 5 | `CAND_OFFICE_ST` | State (2-letter postal code) | `NC` |
| 6 | `CAND_OFFICE` | Office: `H` / `S` / `P` | `H` |
| 7 | `CAND_OFFICE_DISTRICT` | Congressional district (`00` for Senate/President) | `09` |
| 8 | `CAND_ICI` | Incumbent/Challenger/Open: `I`/`C`/`O` | `C` |
| 9 | `CAND_STATUS` | Status code (`C`=statutory candidate, `F`=filed, `N`=not yet, `P`=prior cycle) | `C` |
| 10 | `CAND_PCC` | Principal campaign committee ID | `C00654321` |
| 11 | `CAND_ST1` | Mailing address street | |
| 12 | `CAND_ST2` | Mailing address street 2 | |
| 13 | `CAND_CITY` | Mailing address city | |
| 14 | `CAND_ST` | Mailing address state | |
| 15 | `CAND_ZIP` | Mailing address ZIP | |

## Usage in the pipeline

FEC data serves two purposes:

1. **Stable identifiers.** `CAND_ID` persists across election cycles. A candidate who runs for the same seat in 2020 and 2022 keeps the same ID. This provides a ground-truth link for validating temporal chains built by the L4 layer.

2. **Name cross-referencing.** `CAND_NAME` is parsed at L1 into last, first, middle, and suffix components. These parsed names are compared against MEDSL and state source names during L3 entity resolution. FEC uses `LAST, FIRST MIDDLE` format consistently, which makes it one of the more predictable sources for name parsing.

## Limitations

- **Federal candidates only.** No state legislators, no county commissioners, no school board members. FEC has no jurisdiction over non-federal offices.
- **Filing ≠ appearing on ballot.** Many `CAND_ID` entries correspond to candidates who filed paperwork but never appeared on a general election ballot.
- **Party codes differ from other sources.** FEC uses codes like `LIB`, `GRE`, `NNE` (None) that do not match MEDSL's `LIBERTARIAN`, `GREEN`, `NONPARTISAN` labels. Normalization is required at L1.