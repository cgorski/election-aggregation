# The Project Does Not Store Data

This project processes election data. It does not redistribute it.

## Why Not

### Legal

Each source publishes data under its own terms. MEDSL uses CC-BY. NC SBE publishes as public record under North Carolina law. OpenElections uses a mix of licenses depending on the state contributor. FEC data is public domain. Census reference files are public domain.

Bundling data from all sources into a single download would require compliance with every license simultaneously — attribution chains, share-alike provisions, and restrictions that vary by state contributor. The legal surface area grows with every source added. We avoid it entirely by not storing the data.

### Practical

The current corpus is 8+ GB across three election cycles and seven sources. Adding MEDSL 2018 and 2020, full OpenElections coverage, and VEST shapefiles pushes this past 20 GB. Hosting, versioning, and serving that volume adds infrastructure cost and maintenance burden that contribute nothing to the pipeline's accuracy or reproducibility.

### Freshness

Sources update. NC SBE reissues precinct files when canvass corrections are made. MEDSL publishes errata and revised datasets. OpenElections contributors fix parsing errors and add new states. A copy of the data taken on March 18 may be stale by April 1.

If we store data, every downstream user inherits our staleness. If users download from the authoritative source, they get the latest version — and our pipeline processes it identically.

## What We Provide Instead

The project provides everything needed to acquire the data yourself:

| What | Where | Example |
|------|-------|---------|
| Exact source URLs | Each source chapter in Part II | `https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2022_11_08/results_pct_20221108.zip` |
| Download commands | [Download the Data](../usage/download.md) | `curl -O <url>` with expected file sizes |
| Schema documentation | Each source chapter | Column names, types, delimiters, encoding |
| Known quirks | Each source chapter | NC SBE uses `\t` separators but `.txt` extension; MEDSL 2022 has trailing commas in some state files |
| File size expectations | [Download the Data](../usage/download.md) | MEDSL 2022 NC: ~45 MB compressed |
| SHA-256 of our L0 copies | L0 manifests | Verify your download matches ours |

The L0 manifest for each file records the SHA-256 hash of the bytes we processed. After downloading the same file, you can hash your copy and compare. If the hashes match, your pipeline run will produce identical L1 output — byte for byte, hash for hash.

## The Boundary

The project *does* bundle small reference datasets that are not election results:

- **FIPS code reference files** (~200 KB) from the Census Bureau, public domain. These change only on decennial redistricting.
- **The nickname dictionary** (~5 KB), original to this project.
- **The office classification keyword and regex tables** (~10 KB), original to this project.
- **The 200-name office embedding reference set** (~50 KB), original to this project.

These are small, stable, and authored by the project. They are not third-party election data.

Election results — the 42 million rows of precinct-level vote counts — are never stored, cached, or redistributed. The user downloads them. The pipeline processes them. The outputs live on the user's machine.