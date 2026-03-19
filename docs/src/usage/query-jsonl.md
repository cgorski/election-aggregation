# Querying JSONL Output

Every layer of the pipeline produces JSONL — one JSON record per line. This format is streamable, greppable, and works with standard Unix tools. No database required.

## Format Basics

Each line is a complete, self-contained JSON object:

```text
{"election_date":"2022-11-08","state":"NC","county":"COLUMBUS","candidate_canonical":"Timothy Lance","votes_total":303}
{"election_date":"2022-11-08","state":"NC","county":"COLUMBUS","candidate_canonical":"Bessie Blackwell","votes_total":277}
```

Line count equals record count:

```sh
wc -l l4_canonical/exports/flat_export.jsonl
# 42381902 l4_canonical/exports/flat_export.jsonl
```

## Querying with jq

[jq](https://stedolan.github.io/jq/) is the standard tool for command-line JSON processing. Every example below operates on L4 flat export JSONL.

### Filter by state

```sh
cat flat_export.jsonl | jq -c 'select(.state == "NC")' | head -3
```

Output:

```text
{"election_date":"2022-11-08","state":"NC","county":"COLUMBUS","candidate_canonical":"Timothy Lance","votes_total":303,...}
{"election_date":"2022-11-08","state":"NC","county":"COLUMBUS","candidate_canonical":"Bessie Blackwell","votes_total":277,...}
{"election_date":"2022-11-08","state":"NC","county":"COLUMBUS","candidate_canonical":"Nicky Wooten","votes_total":218,...}
```

### Filter by office level

```sh
cat flat_export.jsonl | jq -c 'select(.contest.office_level == "school_district")' | wc -l
# 1847302
```

### Extract specific fields

```sh
cat flat_export.jsonl \
  | jq -c 'select(.state == "NC" and .county == "COLUMBUS") | {name: .candidate_canonical, votes: .votes_total, office: .contest_name}' \
  | head -5
```

Output:

```text
{"name":"Timothy Lance","votes":303,"office":"COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02"}
{"name":"Bessie Blackwell","votes":277,"office":"COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02"}
{"name":"Nicky Wooten","votes":218,"office":"COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02"}
{"name":"Ricky Leinwand","votes":1531,"office":"COLUMBUS COUNTY SHERIFF"}
{"name":"Jody Greene","votes":1204,"office":"COLUMBUS COUNTY SHERIFF"}
```

### Count distinct candidates per state

```sh
cat flat_export.jsonl \
  | jq -r '.state + "\t" + .candidate_entity_id' \
  | sort -u \
  | cut -f1 \
  | uniq -c \
  | sort -rn \
  | head -5
```

Output:

```text
  14203 TX
  12847 CA
   9341 FL
   7892 NY
   6204 OH
```

### Find all records for a specific candidate

```sh
cat flat_export.jsonl \
  | jq -c 'select(.candidate_entity_id == "person:nc:columbus:lance-timothy-13")' \
  | jq '{precinct: .jurisdiction.precinct, votes: .votes_total}'
```

Output (one line per precinct):

```text
{"precinct":"P17","votes":303}
{"precinct":"P21","votes":287}
{"precinct":"P04","votes":214}
...
```

## Querying with Python

For aggregation, sorting, or anything beyond filtering, Python is more practical.

### Load and filter

```python
import json

with open("flat_export.jsonl") as f:
    nc_school = [
        json.loads(line) for line in f
        if '"NC"' in line  # fast pre-filter on raw text
        and json.loads(line).get("contest", {}).get("office_level") == "school_district"
    ]

print(f"{len(nc_school)} NC school district records")
```

### Stream large files without loading into memory

```python
import json

def stream_jsonl(path, predicate):
    with open(path) as f:
        for line in f:
            record = json.loads(line)
            if predicate(record):
                yield record

for r in stream_jsonl("flat_export.jsonl", lambda r: r["state"] == "NC" and r["votes_total"] > 1000):
    print(r["candidate_canonical"], r["votes_total"], r["contest_name"])
```

### Aggregate to contest level

```python
import json
from collections import defaultdict

totals = defaultdict(lambda: defaultdict(int))

with open("flat_export.jsonl") as f:
    for line in f:
        r = json.loads(line)
        if r["state"] == "NC" and r["county"] == "COLUMBUS":
            key = (r["contest_name"], r["candidate_canonical"])
            totals[r["contest_name"]][r["candidate_canonical"]] += r["votes_total"]

for contest, candidates in sorted(totals.items()):
    print(f"\n{contest}")
    for name, votes in sorted(candidates.items(), key=lambda x: -x[1]):
        print(f"  {name}: {votes:,}")
```

### Export to CSV

```python
import json, csv

with open("flat_export.jsonl") as f_in, open("output.csv", "w", newline="") as f_out:
    writer = csv.writer(f_out)
    writer.writerow(["state", "county", "contest", "candidate", "votes"])
    for line in f_in:
        r = json.loads(line)
        writer.writerow([r["state"], r["county"], r["contest_name"],
                         r["candidate_canonical"], r["votes_total"]])
```

## Five Useful One-Liners

**1. Total votes per state (top 10):**

```sh
jq -r .state flat_export.jsonl | sort | uniq -c | sort -rn | head -10
```

**2. All uncontested races (single candidate per contest):**

```sh
jq -r '"\(.state)\t\(.county)\t\(.contest_name)\t\(.candidate_entity_id)"' flat_export.jsonl \
  | sort -u | cut -f1-3 | uniq -c | awk '$1 == 1' | wc -l
```

**3. Highest single-precinct vote total:**

```sh
jq -c 'select(.votes_total > 50000) | {name: .candidate_canonical, votes: .votes_total, state: .state}' flat_export.jsonl \
  | sort -t: -k2 -rn | head -5
```

**4. Candidates appearing in multiple elections (career tracking):**

```sh
jq -r '"\(.candidate_entity_id)\t\(.election_date)"' flat_export.jsonl \
  | sort -u | cut -f1 | uniq -c | awk '$1 >= 3' | wc -l
# 702
```

**5. Verify a specific hash chain link:**

```sh
jq -c 'select(.l3_hash == "28183d41d50204d5")' l3_matched/nc/2022/matched.jsonl
```

## Performance Notes

- **Streaming is mandatory at scale.** The full L1 corpus at 200 million records is approximately 400 GB of JSONL. Do not load it into memory. Use `jq` with streaming or Python generators.
- **Pre-filter with grep.** For large files, `grep '"NC"' flat_export.jsonl | jq ...` is faster than `jq 'select(.state == "NC")'` alone, because grep uses optimized byte scanning while jq parses every line.
- **Partition files help.** The pipeline stores L1–L3 output partitioned by `{state}/{year}/`. Query a single state-year partition instead of the full national file when possible.
- **For heavy analysis, load into DuckDB or SQLite.** Both can ingest JSONL directly and provide SQL query capabilities with proper indexing.