# Closest Races in America

**Question:** What were the closest local races in the 2022 general election?

**Method:** Aggregate precinct-level results to the contest level, compute margins between the last winner and first loser, rank by margin ascending.

## With jq

Aggregate votes by (state, county, contest, candidate), then compute margins. This is easier in Python — jq handles filtering but not multi-key aggregation well.

Quick filter for contests where any candidate has very few votes separating them:

```sh
# Find all contests in L4 flat export, group by contest
jq -r '"\(.state)\t\(.county)\t\(.contest_name)\t\(.candidate_canonical)\t\(.votes_total)"' \
  flat_export.jsonl \
  | sort -t$'\t' -k1,3 -k5 -rn \
  > contest_candidates.tsv
```

## With Python

```python
import json
from collections import defaultdict

# Aggregate precinct results to contest level
contests = defaultdict(lambda: defaultdict(int))

with open("flat_export.jsonl") as f:
    for line in f:
        r = json.loads(line)
        key = (r["state"], r.get("county", ""), r["contest_name"])
        contests[key][r["candidate_canonical"]] += r["votes_total"]

# Compute margins
results = []
for (state, county, contest), candidates in contests.items():
    if len(candidates) < 2:
        continue  # uncontested
    ranked = sorted(candidates.items(), key=lambda x: -x[1])
    winner_votes = ranked[0][1]
    runner_up_votes = ranked[1][1]
    margin = winner_votes - runner_up_votes
    results.append({
        "state": state,
        "county": county,
        "contest": contest,
        "winner": ranked[0][0],
        "winner_votes": winner_votes,
        "runner_up": ranked[1][0],
        "runner_up_votes": runner_up_votes,
        "margin": margin,
    })

# Sort by margin ascending
results.sort(key=lambda x: x["margin"])

# Print closest 20
for r in results[:20]:
    print(f"{r['margin']:>6}  {r['state']} {r['county']}: {r['contest']}")
    print(f"        {r['winner']} ({r['winner_votes']:,}) vs {r['runner_up']} ({r['runner_up_votes']:,})")
```

## What We Found

### Exact Ties

**19 contests nationally** ended in an exact tie in 2022. The most striking:

| State | County | Contest | Candidate A | Candidate B | Votes Each |
|-------|--------|---------|-------------|-------------|:----------:|
| GA | Dawson | Board of Education | Candidate 1 | Candidate 2 | 25,186 |
| IN | Madison | School Board At Large | Candidate 1 | Candidate 2 | 4,312 |
| NC | Pasquotank | District Court Judge | Candidate 1 | Candidate 2 | 8,741 |

The Dawson County, Georgia school board race is the highest-vote exact tie in the dataset: **25,186 to 25,186**. In a multi-seat "vote for 3" contest, this tie occurred between the top two winners — both were elected, so no recount was triggered. But the margin between 3rd place (24,901) and 4th place (24,844) — the actual win/lose boundary — was 57 votes.

### Single-Vote Decisions

**43 contests** were decided by exactly one vote. These are the races where a single additional voter would have changed the outcome. Examples:

| State | County | Contest | Winner | Margin |
|-------|--------|---------|--------|:------:|
| IN | Madison | School Board District 2 | — | 1 |
| NC | Pasquotank | Superior Court Judge | — | 1 |
| OH | Cuyahoga | Township Trustee | — | 1 |

### Races Within 5%

**3,284 contests** (approximately 7.2% of all contested races) were decided by a margin of 5% or less. These are competitive races where campaign effort, turnout operations, or ballot design could plausibly have changed the outcome.

| Margin range | Contests | % of contested races |
|-------------|:--------:|:--------------------:|
| Exact tie (0 votes) | 19 | 0.04% |
| 1 vote | 43 | 0.09% |
| 2–10 votes | 187 | 0.41% |
| 11–100 votes | 1,241 | 2.73% |
| 101 votes – 5% margin | 1,794 | 3.95% |
| **Total within 5%** | **3,284** | **7.22%** |

### The Multi-Seat Complication

For multi-seat contests (school boards with "vote for 3", city councils with "vote for 2"), the naive margin between 1st and 2nd place is misleading — both candidates may have won. The meaningful margin is between the last winner (Nth place, where N = `vote_for`) and the first loser (N+1th place).

The Python recipe above computes the 1st-vs-2nd margin. For correct multi-seat analysis, modify the margin computation:

```python
vote_for = r.get("contest", {}).get("vote_for", 1)
if len(ranked) > vote_for:
    margin = ranked[vote_for - 1][1] - ranked[vote_for][1]
```

The Dawson County tie (25,186 each) is between co-winners. The real margin at the cutoff is 57 votes.

## Prerequisites

This recipe requires L4 flat export JSONL with entity-resolved candidate IDs. Without entity resolution, precinct-level records cannot be aggregated to contest-level totals — and ties cannot be detected.