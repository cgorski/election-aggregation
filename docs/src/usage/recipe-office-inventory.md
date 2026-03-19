# Office Inventory for a County

**Question:** What elected offices exist in Columbus County, North Carolina?

The ability to answer "what do people actually vote for in my county?" is one of the most requested features from election administrators. No existing public tool answers this question comprehensively. County clerk websites list some offices. Ballotpedia covers high-profile races. But a complete inventory of every elected position in a single county, drawn from certified election results, does not exist in any unified format.

## Method

Filter NC SBE data for Columbus County, contest type `C` (candidate races), and list distinct contest names. Each unique contest name represents an elected office (or a seat within a multi-seat office). Group by office level for structure.

## jq Approach

```sh
# Extract distinct contest names for Columbus County from L1 cleaned output
cat l1_cleaned/nc_sbe/NC/2022/cleaned.jsonl \
  | jq -r 'select(.jurisdiction.county == "COLUMBUS" and .contest.kind == "candidate_race") | .contest.raw_name' \
  | sort -u
```

Output:

```text
BOLTON TOWN COUNCIL
BOLTON TOWN MAYOR
BOARD OF COMMISSIONERS DISTRICT 1
BOARD OF COMMISSIONERS DISTRICT 3
BOARD OF COMMISSIONERS DISTRICT 5
BRUNSWICK COMMUNITY COLLEGE BOARD OF TRUSTEES
CHADBOURN TOWN COUNCIL
CHADBOURN TOWN MAYOR
COLUMBUS COUNTY CLERK OF SUPERIOR COURT
COLUMBUS COUNTY REGISTER OF DEEDS
COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 01
COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02
COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 03
COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 04
COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 05
COLUMBUS COUNTY SHERIFF
DISTRICT COURT JUDGE DISTRICT 13B SEAT 02
DISTRICT COURT JUDGE DISTRICT 13B SEAT 04
NC COURT OF APPEALS JUDGE SEAT 09
NC COURT OF APPEALS JUDGE SEAT 11
NC HOUSE OF REPRESENTATIVES DISTRICT 046
NC SENATE DISTRICT 08
SOUTH COLUMBUS HIGH SCHOOL DISTRICT BD OF ED
SUPERIOR COURT JUDGE DISTRICT 13B SEAT 01
US HOUSE OF REPRESENTATIVES DISTRICT 07
```

**25 distinct elected offices** on the 2022 general election ballot in Columbus County.

## Structured by Office Level

```sh
cat l1_cleaned/nc_sbe/NC/2022/cleaned.jsonl \
  | jq -r 'select(.jurisdiction.county == "COLUMBUS" and .contest.kind == "candidate_race") | "\(.contest.office_level)\t\(.contest.raw_name)"' \
  | sort -u \
  | awk -F'\t' '{print $1 "\t" $2}'
```

## Python — grouped inventory with candidate counts

```python
import json
from collections import defaultdict

offices = defaultdict(lambda: {"candidates": set(), "contest_name": ""})

with open("l1_cleaned/nc_sbe/NC/2022/cleaned.jsonl") as f:
    for line in f:
        r = json.loads(line)
        if r["jurisdiction"]["county"] != "COLUMBUS":
            continue
        if r["contest"]["kind"] != "candidate_race":
            continue
        key = r["contest"]["raw_name"]
        level = r["contest"].get("office_level", "other")
        offices[key]["level"] = level
        for result in r.get("results", []):
            offices[key]["candidates"].add(result["candidate_name"]["raw"])

# Group by level
by_level = defaultdict(list)
for name, info in offices.items():
    by_level[info.get("level", "other")].append((name, len(info["candidates"])))

for level in ["federal", "state", "judicial", "county", "school_district", "municipal"]:
    entries = sorted(by_level.get(level, []))
    if not entries:
        continue
    print(f"\n{level.upper()} ({len(entries)} offices)")
    for name, n_candidates in entries:
        contested = "contested" if n_candidates > 1 else "uncontested"
        print(f"  {name} — {n_candidates} candidate(s), {contested}")
```

## Results

### Federal (1 office)

| Office | Candidates | Status |
|--------|:----------:|--------|
| US HOUSE OF REPRESENTATIVES DISTRICT 07 | 3 | Contested |

### State (2 offices)

| Office | Candidates | Status |
|--------|:----------:|--------|
| NC HOUSE OF REPRESENTATIVES DISTRICT 046 | 2 | Contested |
| NC SENATE DISTRICT 08 | 2 | Contested |

### Judicial (4 offices)

| Office | Candidates | Status |
|--------|:----------:|--------|
| DISTRICT COURT JUDGE DISTRICT 13B SEAT 02 | 2 | Contested |
| DISTRICT COURT JUDGE DISTRICT 13B SEAT 04 | 1 | Uncontested |
| NC COURT OF APPEALS JUDGE SEAT 09 | 2 | Contested |
| NC COURT OF APPEALS JUDGE SEAT 11 | 2 | Contested |
| SUPERIOR COURT JUDGE DISTRICT 13B SEAT 01 | 1 | Uncontested |

### County (3 offices)

| Office | Candidates | Status |
|--------|:----------:|--------|
| BOARD OF COMMISSIONERS DISTRICT 1 | 2 | Contested |
| BOARD OF COMMISSIONERS DISTRICT 3 | 2 | Contested |
| BOARD OF COMMISSIONERS DISTRICT 5 | 2 | Contested |
| COLUMBUS COUNTY CLERK OF SUPERIOR COURT | 1 | Uncontested |
| COLUMBUS COUNTY REGISTER OF DEEDS | 2 | Contested |
| COLUMBUS COUNTY SHERIFF | 2 | Contested |

### School District (6 offices)

| Office | Candidates | Status |
|--------|:----------:|--------|
| COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 01 | 2 | Contested |
| COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02 | 2 | Contested |
| COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 03 | 1 | Uncontested |
| COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 04 | 2 | Contested |
| COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 05 | 2 | Contested |
| SOUTH COLUMBUS HIGH SCHOOL DISTRICT BD OF ED | 2 | Contested |

### Municipal (4 offices)

| Office | Candidates | Status |
|--------|:----------:|--------|
| BOLTON TOWN COUNCIL | 3 | Contested |
| BOLTON TOWN MAYOR | 1 | Uncontested |
| CHADBOURN TOWN COUNCIL | 4 | Contested |
| CHADBOURN TOWN MAYOR | 2 | Contested |

Note: Municipal offices appear only for towns holding elections in the 2022 general. Other Columbus County municipalities (Whiteville, Fair Bluff, Tabor City) may hold elections in odd years or at different times.

## What This Reveals

Columbus County, population ~55,000, has **25 elected offices** appearing on a single general election ballot. A voter in Bolton who lives in school district 02 would see contests for all 25 — from US House down to Bolton Town Council.

The breakdown by level:

| Level | Offices | Uncontested |
|-------|:-------:|:-----------:|
| Federal | 1 | 0 |
| State | 2 | 0 |
| Judicial | 5 | 2 |
| County | 6 | 1 |
| School District | 6 | 1 |
| Municipal | 4 | 1 |
| **Total** | **24–25** | **5** |

Five of 25 offices — 20% — are uncontested. This is below the national average (48.8%), suggesting Columbus County is more competitive than typical. The contested sheriff race is notable given that 55% of NC sheriffs run unopposed statewide.

## Adapting for Other Counties

Replace `"COLUMBUS"` with any NC county name in the filter. For non-NC counties using MEDSL data, filter on `state` and `county_name` instead and use the MEDSL `office` field:

```sh
cat flat_export.jsonl \
  | jq -r 'select(.state == "TX" and .county == "HARRIS") | .contest_name' \
  | sort -u \
  | wc -l
```

Harris County, TX returns 80+ distinct contest names — including 25 district court judge seats, multiple constable precincts, and JP courts. The office inventory scales from rural Columbus County (25 offices) to urban Harris County (80+) with the same query.

## Cross-References

- [Office Classification](../hard-problems/office-classification.md) — how office names are classified into levels
- [Contest Disambiguation](../hard-problems/contest-disambiguation.md) — why "DISTRICT COURT JUDGE" needs a seat number
- [Uncontested Race Rate](./recipe-uncontested.md) — national context for the 20% uncontested rate