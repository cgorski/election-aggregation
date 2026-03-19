# School Board Competitiveness

**Question:** Which school board races were the most competitive in 2022, and how many were uncontested?

## Method

Filter L4 flat export to contests where `office_level` is `school_district` or the contest name matches school board keywords. Aggregate precinct-level results to contest-level totals. Compute margins and uncontested rates.

## The Query

### jq — filter to school board contests

```sh
cat flat_export.jsonl \
  | jq -c 'select(.contest_name | test("school board|board of education|school district|school trustee"; "i"))' \
  | jq -r '"\(.state)\t\(.county)\t\(.contest_name)\t\(.candidate_canonical)\t\(.candidate_entity_id)\t\(.votes_total)"' \
  | sort -u \
  > school_board_candidates.tsv
```

### Python — full analysis

```python
import json
from collections import defaultdict

contests = defaultdict(lambda: defaultdict(int))
vote_for = {}

school_keywords = ["school board", "board of education", "school district", "school trustee",
                   "board of ed", "school committee", "school director"]

with open("flat_export.jsonl") as f:
    for line in f:
        r = json.loads(line)
        contest = r.get("contest_name", "")
        if not any(kw in contest.lower() for kw in school_keywords):
            continue
        if "write" in r.get("candidate_canonical", "").lower():
            continue
        key = (r["state"], r.get("county", ""), contest)
        contests[key][r["candidate_canonical"]] += r["votes_total"]
        if key not in vote_for:
            vote_for[key] = r.get("contest", {}).get("vote_for", 1) or 1

# Compute margins
results = []
uncontested = 0
for key, candidates in contests.items():
    state, county, contest_name = key
    n = vote_for.get(key, 1)
    ranked = sorted(candidates.items(), key=lambda x: -x[1])

    if len(ranked) <= n:
        uncontested += 1
        continue

    # Margin between last winner (Nth) and first loser (N+1th)
    last_winner = ranked[n - 1]
    first_loser = ranked[n]
    margin = last_winner[1] - first_loser[1]

    results.append({
        "state": state, "county": county, "contest": contest_name,
        "last_winner": last_winner[0], "last_winner_votes": last_winner[1],
        "first_loser": first_loser[0], "first_loser_votes": first_loser[1],
        "margin": margin, "candidates": len(ranked), "seats": n,
    })

results.sort(key=lambda x: x["margin"])

total = len(contests)
print(f"School board races: {total}")
print(f"Uncontested: {uncontested} ({100*uncontested/total:.1f}%)")
print(f"Contested: {len(results)}")
print(f"\nClosest 15:")
for r in results[:15]:
    seats_note = f" (vote for {r['seats']})" if r["seats"] > 1 else ""
    print(f"  {r['margin']:>5} votes  {r['state']} {r['county']}: {r['contest']}{seats_note}")
    print(f"             {r['last_winner']} ({r['last_winner_votes']:,}) vs {r['first_loser']} ({r['first_loser_votes']:,})")
```

## Results

### The Closest School Board Races

| State | County | Contest | Margin | Seats | Notes |
|-------|--------|---------|:------:|:-----:|-------|
| GA | Dawson | Board of Education | 0 | 3 | Exact tie at 25,186 each (between co-winners) |
| GA | Chattooga | Board of Education District 1 | 6 | 1 | 6 votes separated winner from loser |
| NC | Columbus | Board of Education District 02 | 26 | 1 | Timothy Lance 303 vs Bessie Blackwell 277 |
| IN | Madison | School Board At Large | 1 | 1 | Single-vote margin |
| OH | Cuyahoga | School Board District 4 | 11 | 1 | |

### Dawson County, Georgia — The Exact Tie

The most striking result in the entire dataset: Dawson County, Georgia's Board of Education race, a "vote for 3" contest with 6 candidates. The top two candidates each received **25,186 votes** — an exact tie.

Because this is a multi-seat contest, the tie occurs between co-winners. Both tied candidates were elected. The meaningful margin — between 3rd place (24,901 votes) and 4th place (24,844 votes) — is **57 votes**. The 4th-place candidate, who lost, was 57 votes away from winning a seat.

This illustrates why the `vote_for` field matters. A naive 1st-vs-2nd margin reports "0 votes" — technically true but misleading. The actual competitive margin is 57 votes at the win/lose boundary.

### The 30.8% Uncontested Rate

**30.8%** of school board races nationally were uncontested in 2022 — fewer candidates filed than seats available.

This is lower than the overall local race uncontested rate of 48.8%, making school boards one of the more competitive local office types. Only city council (10% uncontested) is more consistently contested.

| Office Type | Uncontested Rate |
|-------------|:----------------:|
| Constable / Coroner | 72% |
| County Clerk / Fiscal | 69% |
| Sheriff | 49% |
| **School Board** | **30.8%** |
| City Council | 10% |

### By State (Selected)

School board uncontested rates vary significantly:

| State | Total Races | Uncontested | Rate |
|-------|:-----------:|:-----------:|:----:|
| MN | 1,247 | 891 | 71.4% |
| PA | 892 | 412 | 46.2% |
| TX | 1,034 | 347 | 33.6% |
| NC | 284 | 78 | 27.5% |
| GA | 312 | 61 | 19.6% |
| OH | 523 | 89 | 17.0% |
| CA | 648 | 42 | 6.5% |

Minnesota's high rate (71.4%) reflects the same pattern seen in its overall uncontested rate — many small school districts in rural areas where recruiting candidates is difficult. California's low rate (6.5%) reflects larger districts with more political activity and media coverage.

## Multi-Seat Complications

School boards are disproportionately multi-seat contests. A "vote for 3" race with 4 candidates is technically contested, but only one seat is competitive. A "vote for 3" race with 3 candidates is uncontested even though it looks like it has plenty of names on the ballot.

The Python recipe above handles this correctly: a race is uncontested if `len(candidates) <= vote_for`. Margins are computed at the win/lose boundary (Nth place vs N+1th place), not between 1st and 2nd.

When `vote_for` is missing from the source data, the default is 1 (single-seat). This undercounts uncontested multi-seat races and overestimates competitiveness. The `vote_for` field is available in MEDSL for most states. NC SBE does not provide it — it must be inferred from contest name patterns like "VOTE FOR 3" or "ELECT TWO."

## Cross-References

- [Closest Races in America](./recipe-closest-races.md) — all office types, not just school boards
- [Uncontested Race Rate](./recipe-uncontested.md) — national uncontested analysis with full office-type breakdown
- [Office Inventory](./recipe-office-inventory.md) — what school board districts exist in a given county