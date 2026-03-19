# Contest Kinds: CandidateRace, BallotMeasure, TurnoutMetadata

Every record in the pipeline belongs to exactly one of three contest kinds. This is modeled as a type-level enum — not a string field — so that invalid combinations are rejected at compile time rather than discovered at query time.

## Why three kinds

Election data files mix three fundamentally different things in the same tabular format:

1. A candidate running for office and receiving votes.
2. A ballot measure (bond, referendum, constitutional amendment) where voters choose "Yes" or "No."
3. A metadata row recording registered voters or ballots cast for a precinct, masquerading as a contest.

Sources do not distinguish these. MEDSL puts `REGISTERED VOTERS` in the `office` column as if it were a race. NC SBE creates a "contest" called `Registered Voters - Total` with a "candidate" whose vote count is actually the registration total. Florida OpenElections has 6,013 rows where `office = "Registered Voters"` — 67.9% of all non-candidate records in the initial FL load.

If these are not separated at parse time, downstream analysis produces nonsense: "Registered Voters" appears as the most popular candidate in America, "For" shows up as a person's name in entity resolution, and vote totals are inflated by turnout metadata.

## The enum

```
enum ContestKind {
    CandidateRace {
        results: Vec<CandidateResult>,
    },
    BallotMeasure {
        choices: Vec<BallotChoice>,
        measure_type: BallotMeasureType,
        passage_threshold: Option<f64>,
    },
    TurnoutMetadata {
        registered_voters: Option<u64>,
        ballots_cast: Option<u64>,
    },
}
```

Each variant carries different fields. You cannot accidentally attach a `candidate_name` to a ballot measure or a `passage_threshold` to a candidate race.

## CandidateRace

The common case. A person is running for an office and received votes.

| Field | Type | Description |
|-------|------|-------------|
| `results` | `Vec<CandidateResult>` | One entry per candidate in the contest |

Each `CandidateResult` contains:

| Field | Type | Description |
|-------|------|-------------|
| `candidate_name` | `CandidateName` | Decomposed name (raw, first, middle, last, suffix, nickname, canonical_first) |
| `party` | `Party` | Raw string + normalized enum |
| `votes_total` | `u64` | Total votes received |
| `vote_share` | `Option<f64>` | Percentage of total contest votes |
| `vote_counts_by_type` | `VoteCountsByType` | Breakdown: election_day, early, absentee_mail, provisional |

Examples of CandidateRace contests:

- `US SENATE` — federal
- `GOVERNOR` — state
- `COLUMBUS COUNTY SCHOOLS BOARD OF EDUCATION DISTRICT 02` — local
- `SHERIFF` — county

## BallotMeasure

Voters choose between options (typically "For"/"Against" or "Yes"/"No") on a proposition, bond, amendment, or referendum.

| Field | Type | Description |
|-------|------|-------------|
| `choices` | `Vec<BallotChoice>` | One entry per option |
| `measure_type` | `BallotMeasureType` | Bond, amendment, referendum, etc. |
| `passage_threshold` | `Option<f64>` | Required vote share for passage (e.g., 0.60 for a bond requiring 60%) |

Each `BallotChoice` contains:

| Field | Type | Description |
|-------|------|-------------|
| `choice_text` | `String` | "For", "Against", "Yes", "No", or other option text |
| `votes_total` | `u64` | Votes for this choice |
| `vote_share` | `Option<f64>` | Percentage of total votes |

The `BallotMeasureType` enum: `Bond`, `ConstitutionalAmendment`, `Referendum`, `Initiative`, `Recall`, `Retention`, `Levy`, `Advisory`, `Other`.

### Why this prevents name confusion

Without the BallotMeasure variant, the L1 parser would treat "For" and "Against" as candidate names. They would flow into entity resolution at L3, where the system would try to find other elections where "For" ran for office. By assigning ballot measures to their own variant at parse time, the `choice_text` field is never passed to the name decomposition or embedding logic.

Detection at L1 uses two signals:
- The contest name contains keywords: "bond", "amendment", "referendum", "proposition", "measure", "levy", "question".
- The choice values are in the set {"For", "Against", "Yes", "No", "Bonds", "No Bonds"}.

## TurnoutMetadata

Not a contest at all. These rows carry precinct-level registration and turnout counts that sources embed in the results file as pseudo-contests.

| Field | Type | Description |
|-------|------|-------------|
| `registered_voters` | `Option<u64>` | Registered voter count for this precinct |
| `ballots_cast` | `Option<u64>` | Total ballots cast in this precinct |

Source examples that produce TurnoutMetadata records:

| Source | `office` / `Contest Name` value | `candidate` / `Choice` value |
|--------|--------------------------------|------------------------------|
| MEDSL | `REGISTERED VOTERS` | `REGISTERED VOTERS` |
| MEDSL | `BALLOTS CAST - TOTAL` | `BALLOTS CAST` |
| NC SBE | `Registered Voters - Total` | (numeric total in vote column) |
| OpenElections FL | `Registered Voters` | (numeric total) |

Detection at L1: the contest name matches a known set of turnout keywords (`REGISTERED VOTERS`, `BALLOTS CAST`, `BALLOTS CAST - TOTAL`, `BALLOTS CAST - BLANK`). When detected, the vote count is extracted into `registered_voters` or `ballots_cast`, and the record is tagged as TurnoutMetadata rather than CandidateRace.

These extracted turnout values backfill the `turnout` section of other records in the same precinct. Currently, turnout data is populated for less than 5% of records because most MEDSL state files do not include registration count rows.

## Classification at L1

Contest kind assignment happens during L1 parsing — the deterministic layer. No ML, no embeddings, no API calls. The decision tree:

1. Does the contest name match a turnout keyword? → `TurnoutMetadata`
2. Do the choice values match ballot measure patterns ("For"/"Against"/"Yes"/"No")? → `BallotMeasure`
3. Does the contest name contain ballot measure keywords? → `BallotMeasure`
4. Otherwise → `CandidateRace`

This classification is stored in the record and carried through all subsequent layers. L2 embeds only CandidateRace records for entity resolution. L3 matches only CandidateRace records. BallotMeasure and TurnoutMetadata records pass through L2–L4 without modification beyond provenance tracking.