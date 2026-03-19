# Recipes

Seven recipes, each answering a real question about US local elections with copy-paste commands against pipeline output. Every recipe produces concrete numbers from real data.

## The Recipes

| Recipe | Question | Key Finding |
|--------|----------|-------------|
| [Closest Races in America](./recipe-closest-races.md) | What were the closest local races in 2022? | 19 exact ties nationally; Dawson County GA at 25,186 each |
| [Uncontested Race Rate](./recipe-uncontested.md) | What percentage of local races are uncontested? | 48.8% nationally; constable/coroner at 72%, city council at 10% |
| [Sheriff Accountability](./recipe-sheriffs.md) | How many sheriffs ran unopposed? | 55% in NC, 77% in ME, 74% in MT |
| [School Board Competitiveness](./recipe-school-boards.md) | Which school board races were closest? | Dawson County GA exact tie; 30.8% uncontested nationwide |
| [Office Inventory](./recipe-office-inventory.md) | What elected offices exist in a given county? | Columbus County NC: 25 offices across 6 levels |
| [Career Tracking](./recipe-career-tracking.md) | Who has served longest on a local body? | George Dunlap — 6 cycles, Mecklenburg County, 2014–2024 |
| [Verify a Result](./recipe-verify-result.md) | Can I trace a vote count back to the source file? | Hash chain from L4 to L0, verified for all 200 prototype records |

## How to Use These Recipes

Each recipe includes:

1. **The question** — what you are trying to answer.
2. **The method** — which files to query, which fields to filter on, and how to aggregate.
3. **The commands** — `jq` one-liners and/or Python snippets you can copy and run against your L4 output.
4. **The output** — real numbers from our data, so you know what to expect.

All recipes assume you have pipeline output in `local-data/processed/`. Most operate on L4 flat export JSONL (`l4_canonical/exports/flat_export.jsonl`). The career tracking and verification recipes also reference L1–L3 intermediate files.

Recipes that require entity resolution (career tracking, verification) need the full L0–L4 pipeline to have been run. Recipes that only need contest-level aggregation (closest races, uncontested rates, sheriff accountability) can run against L1 output directly — no API keys required.