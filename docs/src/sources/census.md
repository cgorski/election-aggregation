# Census Bureau FIPS Reference Files

The US Census Bureau publishes authoritative FIPS (Federal Information Processing Standards) code files that provide the canonical mapping from numeric codes to geographic entity names. These files are the ground truth for geographic identifiers across the pipeline.

## What it provides

| File | Entity type | Record count | Key columns |
|------|-------------|-------------:|-------------|
| `state.txt` | States + DC + territories | 57 | `STATE`, `STATE_NAME` |
| `national_county2020.txt` | Counties + equivalents | 3,143 | `STATEFP`, `COUNTYFP`, `COUNTYNAME` |
| `national_place2020.txt` | Incorporated places + CDPs | 31,980 | `STATEFP`, `PLACEFP`, `PLACENAME` |
| `national_cousub2020.txt` | County subdivisions | ~36,000 | `STATEFP`, `COUNTYFP`, `COUSUBFP`, `COUSUBNAME` |

## Format

All files are **pipe-delimited** (`|`) plain text with a header row. Encoding is ASCII. Example from the county file:

```
NC|37|037|1026339|Chatham County|H1|A
NC|37|063|1008557|Durham County|H1|A
NC|37|183|1008586|Wake County|H1|A
```

Columns in the county file:

| Column | Description |
|--------|-------------|
| `STATE` | Two-letter postal abbreviation |
| `STATEFP` | Two-digit state FIPS code |
| `COUNTYFP` | Three-digit county FIPS code |
| `COUNTYNS` | ANSI feature code |
| `COUNTYNAME` | Full county name including "County" suffix |
| `CLASSFP` | FIPS class code (`H1` = active county, `H4` = borough, `H6` = parish) |
| `FUNCSTAT` | Functional status (`A` = active) |

The five-digit county FIPS used throughout the pipeline is `STATEFP` + `COUNTYFP` (e.g., `37` + `183` = `37183` for Wake County, NC).

## Download

```
https://www2.census.gov/geo/docs/reference/state.txt
https://www2.census.gov/geo/docs/reference/codes2020/national_county2020.txt
https://www2.census.gov/geo/docs/reference/codes2020/national_place2020.txt
https://www2.census.gov/geo/docs/reference/codes2020/national_cousub2020.txt
```

No API key required. Files are small (under 5 MB total) and rarely change.

## Usage in the pipeline

Census FIPS files are consumed at **L1** for geographic enrichment. When a source record contains a county name but no FIPS code (common in OpenElections and Clarity data), the pipeline joins against the county file to assign the canonical five-digit FIPS. When a source provides a FIPS code but no name, the lookup runs in reverse.

The place file enables resolution of municipal names to FIPS codes — relevant for city council, mayoral, and municipal utility district contests where the jurisdiction is a place, not a county.

FIPS codes serve as the primary geographic join key across all seven data sources. Without them, matching "Wake County" in MEDSL to "WAKE" in NC SBE to "Wake Co." in OpenElections would require fuzzy string matching. With them, it is an exact integer join.