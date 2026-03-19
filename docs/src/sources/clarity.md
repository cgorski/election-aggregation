# Clarity/Scytl ENR

Clarity (now part of Scytl / CivicPlus) powers Election Night Reporting (ENR) websites for over 1,000 US jurisdictions — counties, cities, and some state-level election authorities. Each jurisdiction runs its own Clarity instance, publishing structured results in XML and JSON formats.

## What Clarity provides

Clarity sites are the primary source for **local race results** that no other source captures: school board, city council, municipal judge, fire district commissioner, water board. Many jurisdictions publish precinct-level results with vote mode breakdowns (Election Day, early, absentee, provisional). Results appear on election night and typically remain available for weeks or months before being replaced by the next election cycle.

## Data format

Results are distributed as XML inside ZIP archives. The XML follows a hierarchical structure:

| Element | Description |
|---------|-------------|
| `<ElectionResult>` | Root element. Contains election metadata (name, date, jurisdiction). |
| `<Contest>` | One per race. Attributes include contest name, vote-for count, total ballots. |
| `<Choice>` | One per candidate or ballot measure option within a contest. Includes name, party, total votes. |
| `<VoteType>` | Breakdown by vote method within each choice. Election Day, absentee, early, provisional. |
| `<Precinct>` | Precinct-level results when the jurisdiction publishes at that granularity. |

A single `detailxml.zip` file for a medium-sized county (50 precincts, 30 contests) is typically 200 KB–2 MB uncompressed.

## URL structure

Clarity ENR sites follow a predictable URL pattern:

```
https://results.enr.clarityelections.com/{state}/{jurisdiction}/{electionID}/
```

The underlying data feeds are at:

| Endpoint | Content |
|----------|---------|
| `reports/detailxml.zip` | Full precinct-level XML results |
| `json/en/summary.json` | Lightweight JSON summary (no precinct detail) |
| `Web02/en/summary.html` | Human-readable results page |

Example for Wake County, NC:

```
https://results.enr.clarityelections.com/NC/Wake/115545/reports/detailxml.zip
```

The `{electionID}` is a numeric identifier assigned per election. It is not sequential and cannot be predicted.

## Coverage

- **Jurisdictions**: ~1,000+ counties and municipalities across ~30 states
- **Election types**: general, primary, runoff, special, municipal
- **Granularity**: precinct-level with vote type breakdowns (most jurisdictions)
- **Temporal**: current election cycle only; prior results are removed when new elections are configured

## Why Clarity matters

Clarity is the highest-value source for local races that do not appear in MEDSL, OpenElections, or state portals. A county's Clarity site may be the only machine-readable source for races like:

- School board (non-partisan, no state-level reporting)
- City council (municipal elections, often off-cycle)
- District court judge retention
- Bond referendums and local ballot measures

## Key problems

**URLs are unstable.** The `{electionID}` changes every cycle. Old results are removed without redirect or archive. There is no central index of active Clarity instances. Discovery requires crawling county election office websites for links.

**No published XML schema.** The XML structure is consistent in practice but not formally specified. Minor variations exist across Clarity software versions. Field names and nesting can differ between jurisdictions.

**Candidate names may embed party.** Some jurisdictions format candidate names as `John Smith (REP)` rather than using a separate party field. This requires parsing at L1.

**Ephemeral availability.** Results may disappear weeks after certification when the jurisdiction configures the site for the next election. L0 acquisition must happen promptly after each election.

## Integration status

Clarity is **not yet integrated** in our pipeline. The source module (`src/sources/clarity.rs`) defines the XML schema and URL patterns but does not implement parsing or acquisition. Integration is blocked on building a jurisdiction discovery mechanism and a scheduled acquisition process that captures results before URLs expire.

When integrated, Clarity will feed into L0 as ZIP archives with XML contents, parsed at L1 into the unified schema. The hierarchical Contest → Choice → VoteType structure maps cleanly to our ContestKind model.