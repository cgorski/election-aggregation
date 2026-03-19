# L0: Raw — Byte-Identical Source Preservation

L0 is the foundation of the pipeline. It stores byte-identical copies of every source file alongside a JSON manifest that records how the file was acquired. Nothing at L0 is parsed, cleaned, or transformed. The raw bytes are sacred.

## What L0 Contains

Every source file produces two artifacts:

| Artifact | Purpose | Example |
|----------|---------|---------|
| The file itself | Exact bytes as downloaded | `results_pct_20221108.txt` |
| The manifest sidecar | Acquisition metadata | `results_pct_20221108.txt.manifest.json` |

The manifest records five fields:

```json
{
  "l0_hash": "edfedf2760cfd54f...",
  "source_url": "https://s3.amazonaws.com/dl.ncsbe.gov/ENRS/2022_11_08/results_pct_20221108.zip",
  "retrieval_date": "2026-03-18T14:30:00Z",
  "file_size_bytes": 18023456,
  "format_detected": "tsv"
}
```

- **`l0_hash`** — SHA-256 of the raw file bytes. This is the root of the hash chain. Every downstream record at L1–L4 ultimately traces back to this value.
- **`source_url`** — The exact URL used to retrieve the file. Not a landing page — the direct download link.
- **`retrieval_date`** — ISO 8601 timestamp of when the file was downloaded. Sources update files in place; the retrieval date disambiguates versions.
- **`file_size_bytes`** — Byte count of the raw file after decompression (if the source was a zip archive, this is the size of the extracted file, not the archive).
- **`format_detected`** — The file format as determined by content inspection: `tsv`, `csv`, `xml`, `json`, `fixed_width`.

## Storage Layout

```text
l0_raw/
├── nc_sbe/
│   ├── results_pct_20221108.txt
│   ├── results_pct_20221108.txt.manifest.json
│   ├── results_pct_20201103.txt
│   └── results_pct_20201103.txt.manifest.json
├── medsl/
│   ├── 2022-nc-precinct-general.csv
│   ├── 2022-nc-precinct-general.csv.manifest.json
│   ├── 2022-fl-precinct-general.csv
│   └── 2022-fl-precinct-general.csv.manifest.json
├── openelections/
│   ├── 20221108__fl__general__precinct.csv
│   └── 20221108__fl__general__precinct.csv.manifest.json
└── census/
    ├── national_county2020.txt
    └── national_county2020.txt.manifest.json
```

Files are organized by source, not by state or year. The source is the natural partition because each source has its own parser at L1. A single MEDSL file may contain data for all 50 states; a single NC SBE file contains one election's results for all NC counties. The source directory mirrors the download structure.

## Idempotent Download

Downloading is idempotent. Before fetching a file, the pipeline checks whether an L0 entry already exists with a matching `l0_hash`:

1. If the manifest exists and the file exists and the file's SHA-256 matches the manifest's `l0_hash` → **skip download**. The file is already present and intact.
2. If the manifest exists but the file is missing or the hash does not match → **re-download**. The file was corrupted or deleted.
3. If no manifest exists → **download and create manifest**.

This means running the download step twice produces no network traffic on the second run. It also means the pipeline recovers gracefully from interrupted downloads — a partially written file will fail the hash check and be re-fetched.

## When Sources Change

Some sources update files in place. NC SBE occasionally reissues precinct result files after canvass corrections. MEDSL publishes revised datasets with the same filename.

When a re-download produces different bytes than the stored `l0_hash`, the pipeline does not overwrite the existing L0 entry. Instead:

1. The new file is stored with a versioned name: `results_pct_20221108.v2.txt`.
2. A new manifest is created with the new `l0_hash` and current `retrieval_date`.
3. The old file and manifest are retained unchanged.

All L1–L4 records that reference the old `l0_hash` remain valid. New pipeline runs against the updated file produce new L1–L4 records referencing the new `l0_hash`. Both versions coexist. The `retrieval_date` field distinguishes them.

## The L0 Hash as Root of Trust

The `l0_hash` is the only value in the pipeline that can be independently verified by anyone with access to the source. Download the file from the URL in the manifest. Compute SHA-256. Compare. If the hashes match, the pipeline processed the same bytes you hold.

Every subsequent hash — `l1_hash`, `l2_hash`, `l3_hash`, `l4_hash` — incorporates its parent's hash. The entire chain is anchored to `l0_hash`. If someone modifies the raw file, the L0 hash changes, the L1 hash no longer matches its `l0_parent_hash`, and the verification algorithm reports a break at the L0→L1 boundary.

In our prototype, all 200 hash chains verified from L4 back to L0 with zero broken links. The verification starts here — at the raw bytes.

## What L0 Does Not Do

L0 does not parse, filter, validate, or transform. A TSV file with malformed rows is stored as-is. A CSV file with a trailing BOM is stored as-is. A zip archive is decompressed and the contents stored, but the extraction is mechanical — no character encoding conversion, no line-ending normalization, no column reordering.

Data quality issues are L1's problem. L0's only job is to preserve the exact bytes that the source published, record where they came from, and make them verifiable.