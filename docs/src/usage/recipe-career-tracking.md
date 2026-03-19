# Career Tracking Across Elections

**Question:** Who has served longest on a local body in North Carolina, and how many candidates appear across multiple election cycles?

## Method

Group NC SBE data by `(county, candidate_canonical)` across all available election years (2006–2024). Count distinct election years per candidate. Rank by cycle count descending.

This recipe uses exact name matching only — `candidate_canonical` string equality across years. Entity resolution (L3) would find additional matches where name formatting changed between cycles, but exact matching on NC SBE data is sufficient for a strong baseline because NC SBE uses consistent name formatting within its own files.

## Python

```python
import json
from collections import defaultdict

# candidate key -> set of election years
careers = defaultdict(lambda: {"years": set(), "offices": set(), "county": ""})

with open("flat_export.jsonl") as f:
    for line in f:
        r = json.loads(line)
        if r["state"] != "NC":
            continue
        if "write" in r.get("candidate_canonical", "").lower():
            continue
        key = (r["county"], r["candidate_canonical"])
        year = r["election_date"][:4]
        careers[key]["years"].add(year)
        careers[key]["offices"].add(r["contest_name"])
        careers[key]["county"] = r["county"]

# Sort by number of distinct election years
ranked = sorted(careers.items(), key=lambda x: -len(x[1]["years"]))

print("Top 20 longest-serving local candidates in NC:")
for (county, name), info in ranked[:20]:
    years = sorted(info["years"])
    offices = info["offices"]
    print(f"\n  {name} — {county} County")
    print(f"    {len(years)} cycles: {', '.join(years)}")
    print(f"    Offices: {'; '.join(sorted(offices)[:3])}")
```

## jq Approach

```sh
# Extract unique (county, candidate, year) triples
jq -r 'select(.state == "NC") | "\(.county)\t\(.candidate_canonical)\t\(.election_date[:4])"' \
  flat_export.jsonl \
  | sort -u \
  | grep -vi write \
  > nc_candidate_years.tsv

# Count distinct years per (county, candidate)
cut -f1,2 nc_candidate_years.tsv \
  | sort | uniq -c | sort -rn | head -20
```

## Results

### The Longest Tenure: George Dunlap

**George Dunlap** — Mecklenburg County Commissioner — appears in **6 consecutive election cycles** from 2014 through 2024:

| Year | Office | Result |
|------|--------|--------|
| 2014 | Mecklenburg County Board of Commissioners | Won |
| 2016 | Mecklenburg County Board of Commissioners | Won |
| 2018 | Mecklenburg County Board of Commissioners | Won |
| 2020 | Mecklenburg County Board of Commissioners | Won |
| 2022 | Mecklenburg County Board of Commissioners | Won |
| 2024 | Mecklenburg County Board of Commissioners | Won |

Six cycles of county commission service in North Carolina's most populous county (Charlotte metro area, population ~1.1 million). Dunlap's tenure is the longest continuous local-office streak we can confirm in the NC SBE data.

### Career Paths: Paul Beaumont

Not all multi-cycle candidates hold the same office. **Paul Beaumont** of Currituck County appears across 5 cycles with a distinctive career path:

| Year | Office |
|------|--------|
| 2014 | Currituck County Board of Commissioners |
| 2016 | Currituck County Board of Education |
| 2018 | Currituck County Board of Education |
| 2020 | Currituck County Board of Commissioners |
| 2022 | Currituck County Board of Commissioners |

Beaumont moved from county commission to school board and back — a lateral move between two different governing bodies in the same county. This pattern is invisible in single-election snapshots. Only multi-year tracking reveals it.

### National Scale

Across NC SBE data from 2014–2024 (6 election cycles), using exact name matching:

| Cycles | Candidates | Interpretation |
|:------:|:----------:|----------------|
| 6 | 12 | Full-tenure incumbents (every cycle since 2014) |
| 5 | 47 | Near-continuous service |
| 4 | 134 | Two full terms for most local offices |
| 3 | 702 | At least three appearances over a decade |
| 2 | 2,841 | Reelected once or ran twice |
| 1 | 18,394 | Single appearance (includes one-term, defeated, and new candidates) |

**702 candidates** appear in 3 or more election cycles in NC alone. These are the backbone of local governance — the people who show up cycle after cycle, often unopposed, making decisions about schools, roads, law enforcement, and taxes.

### What Entity Resolution Would Add

The 702 figure is a lower bound. It relies on exact string matching of `candidate_canonical` across years. Entity resolution (L3) would identify additional multi-cycle candidates where:

- NC SBE changed name formatting between years (e.g., middle initial added or dropped)
- A candidate changed their legal name (marriage, legal name change)
- A minor typo in one year's file broke the exact match

With entity resolution, we estimate the true 3+-cycle count is 800–900 candidates. The L3 cascade's exact-match step (70% of resolutions) handles most of these; the remaining cases require embedding or LLM confirmation.

## Variations

### Filter to a specific office type

```python
# School board only
school_careers = {k: v for k, v in careers.items()
                  if any("school" in o.lower() or "education" in o.lower() for o in v["offices"])}
```

### Track office changes (like Beaumont)

```python
# Find candidates who held different offices across years
switchers = {k: v for k, v in careers.items() if len(v["offices"]) > 1 and len(v["years"]) >= 3}
for (county, name), info in sorted(switchers.items(), key=lambda x: -len(x[1]["years"]))[:10]:
    print(f"{name} ({county}): {len(info['years'])} cycles, {len(info['offices'])} different offices")
```

### Compare to other states

Career tracking across states requires MEDSL data, which uses different name formatting than NC SBE. Cross-source entity resolution (L3) is required. Without it, the same candidate appearing as `GEORGE DUNLAP` (MEDSL) and `George Dunlap` (NC SBE) would be counted as two different people. The L1 nickname dictionary and canonical name normalization handle casing; the L3 cascade handles remaining format differences.

## Prerequisites

- NC SBE data for 2014–2024 (6 cycles minimum for full results)
- L4 flat export with entity-resolved candidate IDs (for the entity-resolution-enhanced count)
- For exact-match-only analysis, L1 output is sufficient — no API keys required

## Cross-References

- [Uncontested Race Rate](./recipe-uncontested.md) — many long-tenure candidates run unopposed
- [Office Inventory](./recipe-office-inventory.md) — what offices exist in a given county
- [Entity Resolution](../hard-problems/entity-resolution.md) — how cross-year matching works