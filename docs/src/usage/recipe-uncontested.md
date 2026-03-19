# Uncontested Race Rate by State

**Question:** What percentage of local races are uncontested — only one candidate on the ballot?

## Method

A race is uncontested if exactly one non-write-in candidate filed. Group L4 flat export records by `(state, county, contest_name, election_date)`, count distinct `candidate_entity_id` values excluding write-in placeholders, and flag contests where the count equals 1.

## The Query

### jq — count uncontested contests in a single state

```sh
# Step 1: Extract unique (contest, candidate) pairs, excluding write-ins
jq -r 'select(.state == "NC" and .candidate_canonical != "Write-In") | "\(.state)\t\(.county)\t\(.contest_name)\t\(.candidate_entity_id)"' \
  flat_export.jsonl \
  | sort -u > nc_contest_candidates.tsv

# Step 2: Count candidates per contest
cut -f1-3 nc_contest_candidates.tsv | uniq -c | sort -rn > nc_contest_counts.tsv

# Step 3: Count uncontested (1 candidate) vs contested (2+)
awk '{print ($1 == 1) ? "uncontested" : "contested"}' nc_contest_counts.tsv | sort | uniq -c
```

### Python — national analysis with office-type breakdown

```python
import json
from collections import defaultdict

contests = defaultdict(set)  # (state, county, contest) -> set of candidate IDs
office_levels = {}           # (state, county, contest) -> office_level

with open("flat_export.jsonl") as f:
    for line in f:
        r = json.loads(line)
        if r["candidate_canonical"] in ("Write-In", "WRITE-IN", "Write-in"):
            continue
        key = (r["state"], r["county"], r["contest_name"])
        contests[key].add(r["candidate_entity_id"])
        if key not in office_levels:
            office_levels[key] = r.get("contest", {}).get("office_level", "unknown")

total = len(contests)
uncontested = sum(1 for cands in contests.values() if len(cands) == 1)
print(f"National: {uncontested}/{total} = {uncontested/total:.1%} uncontested")

# By office type
by_office = defaultdict(lambda: {"total": 0, "uncontested": 0})
for key, cands in contests.items():
    level = office_levels.get(key, "unknown")
    by_office[level]["total"] += 1
    if len(cands) == 1:
        by_office[level]["uncontested"] += 1

print("\nBy office type:")
for office, counts in sorted(by_office.items(), key=lambda x: -x[1]["uncontested"]/max(x[1]["total"],1)):
    rate = counts["uncontested"] / counts["total"]
    print(f"  {office:25s} {rate:5.1%}  ({counts['uncontested']:,} / {counts['total']:,})")
```

## Results

### National Rate

**48.8%** of local races in the MEDSL 2022 keyword-classified subset are uncontested. Nearly half of all elected positions in America had only one name on the ballot.

### By Office Type

| Office Type | Uncontested Rate | Notes |
|-------------|:----------------:|-------|
| Constable / Coroner | 72% | Smallest offices; often no one files to run |
| County Clerk / Fiscal Officer | 69% | Administrative roles with low public visibility |
| Sheriff | 49% | See [Sheriff recipe](./recipe-sheriffs.md) for state-by-state detail |
| School Board | 31% | More competitive than most county offices |
| City Council | 10% | Most competitive local office type |

The pattern is consistent: the less visible the office, the less likely someone runs against the incumbent. City council races — the most visible local office, often covered by local media — are contested 90% of the time. Constable races, which most voters cannot name, are uncontested nearly three-quarters of the time.

### By State (Selected)

| State | Uncontested Rate | Notes |
|-------|:----------------:|-------|
| MN | 89.3% | Highest in the nation; many township offices with no challenger |
| MS | 78.1% | |
| AR | 72.4% | |
| SC | 67.2% | |
| GA | 52.1% | |
| NC | 44.7% | |
| TX | 38.9% | |
| OH | 29.4% | |
| CA | 12.3% | |
| FL | 0.0% | Florida law removes uncontested races from the ballot entirely |

**Florida's 0%** is a methodological artifact, not a sign of democratic vigor. Florida statute §101.151 removes candidates with no opposition from the general election ballot — they win automatically in the primary or by default. The MEDSL general election file therefore contains no uncontested races for FL, because they never appeared on the general election ballot. The true uncontested rate in Florida is substantial but can only be measured from primary election data.

Minnesota's 89.3% reflects the state's large number of township-level offices (township supervisors, township clerks, township treasurers) that rarely attract challengers.

## Interpreting the Results

### What "uncontested" means

A race is uncontested in our analysis if exactly one non-write-in candidate appears in the certified results. This does not account for:

- **Candidates who dropped out.** A race with two filers where one withdrew before election day appears contested in our data (two names on the ballot) even though voters had no real choice.
- **Write-in-only opposition.** A race with one official candidate and a write-in candidate receiving 12 votes is "contested" only in a technical sense. We exclude write-ins from the count.
- **Primary competition.** A sheriff with no general election opponent may have faced a contested primary. Our current analysis uses general election data only.

### Why it matters

An uncontested rate of 48.8% means that for nearly half of local elected positions, the outcome was decided before a single vote was cast. Voters in those jurisdictions had no choice to make for those offices — the only name on the ballot won by default.

This is not inherently bad. Some offices are genuinely non-partisan administrative roles where competent incumbents face no opposition because they are doing a good job. But in aggregate, a 48.8% uncontested rate raises questions about democratic participation, candidate recruitment, and whether voters are aware of the offices they are electing.

### Further analysis

- Filter by `vote_for` > 1 for multi-seat races where "uncontested" means fewer candidates than seats.
- Compare uncontested rates across election cycles (2018 vs 2020 vs 2022) using NC SBE multi-year data.
- Cross-reference with turnout data where available — do precincts with many uncontested races have lower turnout?

## Cross-References

- [Sheriff Accountability](./recipe-sheriffs.md) — deep dive into sheriff uncontested rates by state
- [School Board Competitiveness](./recipe-school-boards.md) — school board margins and uncontested rates
- [Office Inventory](./recipe-office-inventory.md) — what offices exist in a given county