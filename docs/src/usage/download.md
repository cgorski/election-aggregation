# Download the Data

This project does not redistribute election data. You download it yourself from the authoritative sources, verify file integrity, and point the pipeline at your local copies.

## Prerequisites

- ~8 GB disk space for the core dataset (MEDSL 2022 + NC SBE 2022)
- ~20 GB for the full dataset (all years, all sources)
- `curl` or `wget` for downloads
- `unzip` for compressed archives
- `sha256sum` (Linux) or `shasum -a 256` (macOS) for verification

## Core Dataset

The minimum dataset to run the pipeline and reproduce prototype results:

### MEDSL 2022 (All States)

The MIT Election Data + Science Lab publishes precinct-level returns for all 50 states and DC.

```text
mkdir -p local-data/sources/medsl/2022
cd local-data/sources/medsl/2022

# Download from Harvard Dataverse (2022 precinct-level general election)
# File: 2022-precinct-general.csv (~2 GB compressed)
curl -L -o 2022-precinct-general.zip \
  "https://dataverse.harvard.edu/api/access/datafile/:persistentId?persistentId=doi:10.7910/DVN/PJ7QWD/VOQCHQ"
unzip 2022-precinct-general.zip
```

Expected size: ~2 GB compressed, ~6 GB uncompressed. Contains approximately 42 million rows across all states. Format: CSV with columns `state`, `county_name`, `jurisdiction`, `office`, `district`, `candidate`, `party_simplified`, `mode`, `votes`, and others.

### NC SBE 2022

The North Carolina State Board of Elections publishes precinct-level results for every NC election.

```text
mkdir -p local-data/sources/ncsbe/2022
cd local-data/sources/ncsbe/2022

curl -O https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2022_11_08/results_pct_20221108.zip
unzip results_pct_20221108.zip
```

Expected size: ~18 MB compressed, ~75 MB uncompressed. Format: TSV (tab-separated, `.txt` extension). Contains precinct-level results for all NC contests in the 2022 general election — federal, state, county, municipal, judicial, and school board.

### NC SBE 2018 + 2020 (For Multi-Year Analysis)

Required for career tracking and temporal chain validation:

```text
mkdir -p local-data/sources/ncsbe/2020
cd local-data/sources/ncsbe/2020
curl -O https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2020_11_03/results_pct_20201103.zip
unzip results_pct_20201103.zip

mkdir -p local-data/sources/ncsbe/2018
cd local-data/sources/ncsbe/2018
curl -O https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2018_11_06/results_pct_20181106.zip
unzip results_pct_20181106.zip
```

Expected size: ~15 MB compressed each.

## Full Dataset

For comprehensive analysis across all supported years and sources:

### MEDSL 2018 + 2020

```text
mkdir -p local-data/sources/medsl/2020
cd local-data/sources/medsl/2020
# Download from Harvard Dataverse (2020 precinct-level general election)
curl -L -o 2020-precinct-general.zip \
  "https://dataverse.harvard.edu/api/access/datafile/:persistentId?persistentId=doi:10.7910/DVN/K7760H/GKWF2X"
unzip 2020-precinct-general.zip

mkdir -p local-data/sources/medsl/2018
cd local-data/sources/medsl/2018
curl -L -o 2018-precinct-general.zip \
  "https://dataverse.harvard.edu/api/access/datafile/:persistentId?persistentId=doi:10.7910/DVN/UBKYRU/EJMDUL"
unzip 2018-precinct-general.zip
```

Expected size: ~2 GB compressed per year.

### NC SBE 2006–2024 (Deep NC History)

For the full 10-cycle career tracking analysis (George Dunlap's 6 consecutive cycles, 702 candidates in 3+ cycles):

```text
for year in 2006 2008 2010 2012 2014 2016; do
  mkdir -p local-data/sources/ncsbe/${year}
  # NC SBE URL pattern varies by year — check https://dl.ncsbe.gov/ENRS/
  # for the exact filename for each election date
done
```

NC SBE files from 2006–2016 use slightly different column layouts than 2018+. The `nc_sbe` parser handles both formats. Total size for all NC SBE years: ~200 MB.

### OpenElections

Community-curated precinct data for select states. Coverage varies by state and contributor.

```text
mkdir -p local-data/sources/openelections/2022
cd local-data/sources/openelections/2022

# Florida 2022 general
curl -O https://raw.githubusercontent.com/openelections/openelections-data-fl/master/2022/20221108__fl__general__precinct.csv

# Ohio 2022 general
curl -O https://raw.githubusercontent.com/openelections/openelections-data-oh/master/2022/20221108__oh__general__precinct.csv
```

Expected sizes: FL ~50 MB, OH ~30 MB. OpenElections data varies in format by state — some use standardized column names, others preserve county clerk formatting. Total across all available states: ~250 MB.

## Expected Sizes Summary

| Source | Years | Compressed | Uncompressed | Records (approx.) |
|--------|-------|:----------:|:------------:|:-----------------:|
| MEDSL | 2022 | ~2 GB | ~6 GB | ~42M |
| MEDSL | 2020 | ~2 GB | ~5.5 GB | ~38M |
| MEDSL | 2018 | ~2 GB | ~5 GB | ~35M |
| NC SBE | 2022 | 18 MB | 75 MB | ~600K |
| NC SBE | 2006–2024 (all) | ~60 MB | ~200 MB | ~4M |
| OpenElections | 2022 (6 states) | ~80 MB | ~250 MB | ~2M |
| **Core dataset** | | **~2 GB** | **~6 GB** | **~42M** |
| **Full dataset** | | **~8 GB** | **~22 GB** | **~120M** |

## Storage Layout

After downloading, your `local-data/` directory should look like:

```text
local-data/
└── sources/
    ├── medsl/
    │   ├── 2018/
    │   │   └── 2018-precinct-general.csv
    │   ├── 2020/
    │   │   └── 2020-precinct-general.csv
    │   └── 2022/
    │       └── 2022-precinct-general.csv
    ├── ncsbe/
    │   ├── 2018/
    │   │   └── results_pct_20181106.txt
    │   ├── 2020/
    │   │   └── results_pct_20201103.txt
    │   └── 2022/
    │       └── results_pct_20221108.txt
    ├── openelections/
    │   └── 2022/
    │       ├── 20221108__fl__general__precinct.csv
    │       └── 20221108__oh__general__precinct.csv
    └── census/
        └── national_county2020.txt
```

The pipeline's L0 step copies files from `local-data/sources/` into `local-data/processed/l0_raw/` with manifest sidecars. Your source directory is never modified.

## Verification

After downloading, verify file sizes against the values above. For exact reproducibility against our prototype results, verify SHA-256 hashes:

```bash
# macOS
shasum -a 256 local-data/sources/ncsbe/2022/results_pct_20221108.txt

# Linux
sha256sum local-data/sources/ncsbe/2022/results_pct_20221108.txt
```

Compare the output against the `l0_hash` values in the L0 manifests produced by the pipeline. If your hash matches our manifest, your pipeline run will produce identical L1 output — byte for byte, hash for hash.

If the hash does not match, the source may have been updated since our retrieval. The pipeline will still process the file correctly — the L0 manifest will record a different `l0_hash` and `retrieval_date`, and the hash chain will be internally consistent. But numerical results may differ from our published prototype values.

## Census Reference Data

FIPS code reference files are small (~200 KB) and bundled with the project. No separate download is needed. They are located at `src/data/` in the repository and loaded automatically during L1 processing.