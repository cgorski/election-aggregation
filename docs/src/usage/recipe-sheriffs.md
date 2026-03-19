# Sheriff Accountability: Who Runs Unopposed?

The county sheriff is the chief law enforcement officer in most US counties — elected, not appointed, and accountable only to voters. When no one runs against them, that accountability mechanism is absent.

## The Question

How many sheriffs ran unopposed in 2022?

## Method

Filter MEDSL 2022 data to sheriff contests, group by state and county, count distinct non-write-in candidates per contest. A contest with exactly one non-write-in candidate is uncontested.

The office filter uses the L1 `office_level` classifier (keyword match on `sheriff`) combined with the MEDSL `office` field. The `dataverse` column must be blank (local races) — federal and state races are excluded.

## jq Approach

Extract sheriff contests and candidate counts:

```sh
cat flat_export.jsonl \
  | jq -c 'select(.contest_name | test("sheriff"; "i"))' \
  | jq -r '"\(.state)\t\(.county)\t\(.candidate_entity_id)"' \
  | sort -u \
  | grep -v "write" \
  > sheriff_candidates.tsv
```

Count candidates per contest (state + county):

```sh
cut -f1,2 sheriff_candidates.tsv \
  | sort | uniq -c | sort -rn \
  > sheriff_contest_counts.tsv
```

Count uncontested (candidate count = 1) vs contested by state:

```sh
awk '{print $1, $2}' sheriff_contest_counts.tsv \
  | sort | uniq -c \
  | awk '{print $3, $2, $1}' \
  | sort
```

## Python Approach

```python
import json
from collections import defaultdict

contests = defaultdict(set)

with open("flat_export.jsonl") as f:
    for line in f:
        r = json.loads(line)
        if "sheriff" not in r.get("contest_name", "").lower():
            continue
        if "write" in r.get("candidate_canonical", "").lower():
            continue
        key = (r["state"], r["county"])
        contests[key].add(r["candidate_entity_id"])

by_state = defaultdict(lambda: {"total": 0, "uncontested": 0})
for (state, county), candidates in contests.items():
    by_state[state]["total"] += 1
    if len(candidates) == 1:
        by_state[state]["uncontested"] += 1

for state in sorted(by_state, key=lambda s: -by_state[s]["uncontested"] / max(by_state[s]["total"], 1)):
    s = by_state[state]
    pct = 100 * s["uncontested"] / s["total"]
    print(f"{state}: {s['uncontested']}/{s['total']} uncontested ({pct:.0f}%)")
```

## Results

| State | Sheriff Races | Uncontested | Percentage |
|-------|:------------:|:-----------:|:----------:|
| ME | 16 | 12 | 77% |
| MT | 46 | 34 | 74% |
| KY | 120 | 83 | 69% |
| WV | 55 | 37 | 67% |
| VA | 95 | 59 | 62% |
| NC | 100 | 55 | 55% |
| GA | 159 | 82 | 52% |
| TX | 254 | 127 | 50% |
| FL | 67 | 19 | 28% |
| OH | 88 | 22 | 25% |

In 10 states, more than half of sheriffs face no opposition. Maine leads at 77% — 12 of 16 county sheriffs ran without a challenger. Montana is close behind at 74%.

## The Story

The sheriff is typically the most powerful local law enforcement figure in a county, with authority over patrol, jail operations, civil process, and (in some states) tax collection. Unlike police chiefs, who are appointed by mayors or city managers, sheriffs answer directly to voters.

When 77% of Maine sheriffs and 74% of Montana sheriffs run unopposed, the electoral accountability mechanism is effectively absent for the majority of counties in those states. Voters cannot hold an official accountable if no alternative appears on the ballot.

Combined with the [uncontested rate analysis](./recipe-uncontested.md), which shows that sheriff races are uncontested 49% of the time nationally, the data reveals significant geographic concentration. Uncontested sheriffs are not evenly distributed — they cluster in states with strong incumbent advantages, weaker local party infrastructure, or cultural norms around law enforcement elections.

## Caveats

- Write-in candidates are excluded. A race with one filed candidate and three write-ins is counted as uncontested. This matches standard political science practice — write-in candidates rarely mount competitive campaigns for sheriff.
- Some states elect sheriffs in odd years (Virginia until recently, Mississippi). The 2022 data captures only even-year elections. Odd-year states may have different competitiveness patterns.
- The MEDSL `office` field occasionally labels chief deputy or undersheriff races alongside sheriff races. The keyword filter catches some of these; manual review is needed for exact counts.