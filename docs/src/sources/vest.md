# VEST — Voting and Election Science Team

The [Voting and Election Science Team (VEST)](https://election.lab.ufl.edu/precinct-data/) publishes precinct-level election shapefiles for all 50 states. Each shapefile pairs precinct geographic boundaries with vote counts encoded as attribute columns. The data is archived on the [Harvard Dataverse](https://dataverse.harvard.edu/dataverse/electionscience).

## What VEST provides

VEST's primary value is twofold: geographic precinct boundaries (polygons) and odd-year election coverage. No other source in our corpus provides precinct geometries, and MEDSL's loaded data currently covers only even years.

We have downloaded VEST shapefiles for KY, LA, MS, and VA covering the 2015 and 2019 odd-year elections. These contain state-level races (governor, attorney general, state legislature) but not local races.

## Data format

Each state-year dataset is a ZIP archive containing a standard ESRI shapefile bundle:

| File | Purpose |
|------|---------|
| `.shp` | Geometry (precinct boundary polygons) |
| `.dbf` | Attribute table (vote counts, FIPS codes) |
| `.shx` | Spatial index |
| `.prj` | Coordinate reference system definition |
| `.cpg` | Character encoding declaration |

Reading requires a spatial data library. In Python, `geopandas.read_file()` handles the full bundle. In Rust, the `shapefile` crate reads `.shp`/`.dbf` pairs.

## Column encoding convention

VEST encodes election metadata into column names using a compact format:

```
{stage}{YY}{office}{party}{surname}
```

| Component | Values | Examples |
|-----------|--------|----------|
| Stage | `G` (general), `P` (primary), `R` (runoff) | `G` |
| Year | Two-digit year | `20`, `19`, `15` |
| Office | `PRE` (President), `USS` (US Senate), `USH` (US House), `GOV` (Governor), `SOS` (Sec. of State), `AG` (Attorney General), `LTG` (Lt. Governor) | `PRE` |
| Party | `R` (Republican), `D` (Democrat), `L` (Libertarian), `G` (Green), `O` (Other) | `R` |
| Surname | Abbreviated (typically 3 chars) | `TRU`, `BID` |

### Decoded examples

| Column | Stage | Year | Office | Party | Candidate |
|--------|-------|------|--------|-------|-----------|
| `G20PRERTRU` | General | 2020 | President | Republican | Trump |
| `G20PREDBID` | General | 2020 | President | Democrat | Biden |
| `G19GOVDBED` | General | 2019 | Governor | Democrat | Beshear (KY) |
| `G15GOVDEDW` | General | 2015 | Governor | Democrat | Edwards (LA) |
| `G18GOVDABO` | General | 2018 | Governor | Democrat | Abrams (GA) |

## Attribute table structure

The `.dbf` file contains both geographic identifiers and vote count columns:

| Column pattern | Description |
|----------------|-------------|
| `STATEFP20` | 2-digit state FIPS code |
| `COUNTYFP20` | 3-digit county FIPS code |
| `VTDST20` | Voting tabulation district (precinct) code |
| `NAME20` | Human-readable precinct name |
| `ALAND20` | Land area in square meters |
| `AWATER20` | Water area in square meters |
| `G20PRE*` | Vote count columns (one per candidate) |

Vote values are raw integer counts. Each row is one precinct.

## dBASE column name truncation

The `.dbf` format (dBASE III) limits column names to 10 characters. This truncation creates ambiguity:

- `G20USSRPER` could be Perdue or Perry
- `G20USHDWIL` could be Williams, Wilson, or Wilkins

VEST documentation files (included in each ZIP) provide a column-to-candidate mapping. These must be consulted to resolve truncated names.

## Coverage in our pipeline

| State | Year | Election type | Races |
|-------|------|---------------|-------|
| KY | 2019 | Governor, AG, SOS, state legislature | State-level only |
| LA | 2015, 2019 | Governor, state legislature | State-level only |
| MS | 2019 | Governor, AG, state legislature | State-level only |
| VA | 2015, 2019 | Governor, state legislature | State-level only |

These four states hold odd-year elections, which MEDSL has on Dataverse but which we have not yet loaded from that source. VEST fills the gap for state-level races in these cycles.

## Limitations

**No local races.** VEST encodes statewide and federal contests only. County commissioner, school board, sheriff, and other local offices are not present. For local race coverage, use MEDSL or state-specific sources.

**Large file sizes.** Individual state shapefiles range from 50 MB to 500+ MB. The geometry data dominates file size; vote counts are a small fraction.

**Precinct boundary instability.** Redistricting changes precinct boundaries between election cycles. A precinct polygon from 2020 may not correspond to the same geographic area in 2022. Cross-year geographic comparisons require spatial intersection, not ID matching.

**Requires spatial tooling.** Unlike CSV sources that can be read with any text processor, shapefiles require `geopandas` (Python) or the `shapefile` crate (Rust). This adds a dependency that other sources do not.

## Usage in the pipeline

VEST data enters at L0 as the raw shapefile ZIP. At L1, the column encoding is decoded to extract year, office, party, and candidate surname. Vote counts are pivoted from wide format (one column per candidate) to long format (one row per candidate per precinct) to match the unified schema.

The geographic boundaries are preserved as sidecar geometry files but are not embedded into the JSONL record stream. They are available for spatial joins and map rendering but are not part of the core election result schema.

## Download

VEST datasets are available from the Harvard Dataverse. Each state-year combination has its own DOI. Example for Kentucky 2019:

```sh
mkdir -p local-data/sources/vest/ky/2019
curl -L -o local-data/sources/vest/ky/2019/ky_2019.zip \
  "https://dataverse.harvard.edu/api/access/dataset/:persistentId/?persistentId=doi:10.7910/DVN/XXXXXX"
unzip local-data/sources/vest/ky/2019/ky_2019.zip -d local-data/sources/vest/ky/2019/
```

Consult the [VEST precinct data page](https://election.lab.ufl.edu/precinct-data/) for current DOIs. File IDs change when datasets are updated.