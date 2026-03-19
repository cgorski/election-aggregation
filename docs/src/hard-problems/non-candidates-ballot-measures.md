# Ballot Measure Choices: For/Against/Yes/No

When a row in an election results file has "For" as the candidate name, it could mean two things: a person whose legal name is "For" (implausible), or a choice on a ballot measure (almost certain). The distinction cannot be made from the candidate name alone — it requires examining the contest name alongside it.

## The Problem

Ballot measures appear in election data using the same schema as candidate races. The "candidate" column holds "For", "Against", "Yes", or "No". The "contest" column holds something like "BOND REFERENDUM - SCHOOL CONSTRUCTION" or "CONSTITUTIONAL AMENDMENT 3". Nothing in the file format distinguishes a ballot measure from a candidate race.

Real examples from MEDSL 2022:

| Contest Name | Candidate Name | Votes | What It Actually Is |
|-------------|---------------|------:|---------------------|
| CONSTITUTIONAL AMENDMENT 1 | For | 1,847,312 | Ballot measure choice |
| BOND REFERENDUM COLUMBUS COUNTY SCHOOLS | Against | 4,219 | Ballot measure choice |
| COUNTY SALES TAX REFERENDUM | Yes | 31,408 | Ballot measure choice |
| CHARTER AMENDMENT - TERM LIMITS | No | 12,773 | Ballot measure choice |

If these rows enter the candidate pipeline, "For" becomes a person entity. "For" then appears in entity resolution, gets a `candidate_entity_id`, and shows up in the L4 canonical export as the most prolific politician in America — winning thousands of races across every state and every office level.

## The L4 Audit Discovery

In our prototype, the L4 LLM entity audit examined 50 entities for plausibility. Among the 4 errors it identified:

> "'For' is not a plausible person name. This entity appears across 347 contests in 12 states, always in contest names containing 'amendment', 'bond', 'referendum', or 'proposition'. These are ballot measure choices, not candidates."

The audit correctly identified the contamination. But detecting it at L4 is too late — the bad entity has already propagated through L2 embeddings and L3 matching. The fix is detection at L1.

## Detection Logic

A candidate name of "For", "Against", "Yes", or "No" is ambiguous in isolation. These are common English words, and while no real candidate in our dataset is named "For", names like "Yes" are theoretically possible. The detection requires both signals:

**Signal 1: Candidate name pattern.** The candidate name is one of a small set of ballot measure choice words:

| Candidate Name | Ballot Measure Choice |
|---------------|:---------------------:|
| For | Yes |
| Against | Yes |
| Yes | Yes |
| No | Yes |
| Bonds Yes | Yes |
| Bonds No | Yes |
| For the Tax Levy | Yes |
| Against the Tax Levy | Yes |

**Signal 2: Contest name pattern.** The contest name contains one or more ballot measure keywords:

| Keyword | Example Contest Name |
|---------|---------------------|
| amendment | CONSTITUTIONAL AMENDMENT 1 |
| bond | BOND REFERENDUM COLUMBUS COUNTY SCHOOLS |
| referendum | COUNTY SALES TAX REFERENDUM |
| proposition | PROPOSITION 30 - TAX ON INCOME |
| measure | MEASURE A - PARCEL TAX |
| initiative | INITIATIVE 82 - TIPPED WAGES |
| question | BALLOT QUESTION 4 |
| charter | CHARTER AMENDMENT - TERM LIMITS |
| levy | RENEWAL 2.0 MILL LEVY - FIRE |
| issue | ISSUE 1 - REPRODUCTIVE RIGHTS |

Both signals must be present. A candidate named "For" in a contest called "COUNTY COMMISSIONER" would not trigger ballot measure detection — it would be flagged as a data quality anomaly for manual review. A candidate named "John Smith" in a contest called "BOND REFERENDUM" is not a ballot measure choice — the candidate name does not match the pattern.

## Routing

When both signals match, L1 routes the record to `BallotMeasure` contest kind with a `MeasureChoice` result type instead of `CandidateResult`:

```json
{
  "contest": {
    "kind": "ballot_measure",
    "raw_name": "BOND REFERENDUM COLUMBUS COUNTY SCHOOLS",
    "office_level": "school_district",
    "measure_type": "bond"
  },
  "results": [
    {
      "measure_choice": "against",
      "votes_total": 4219,
      "vote_counts_by_type": {
        "election_day": 2107,
        "early": 1891,
        "absentee_mail": 198,
        "provisional": 23
      }
    }
  ]
}
```

The `measure_choice` field replaces `candidate_name`. No name decomposition is performed (there is no first, middle, last, or suffix for "Against"). No entity resolution is needed — "For" in one contest is not the same entity as "For" in another contest. No embedding is generated.

## Edge Cases

**"For the Tax Levy" vs "For."** Some sources use complete phrases like "For the Tax Levy" rather than bare "For". The pattern match checks for the prefix, not exact equality.

**Mixed contests.** A small number of records have both candidate names and ballot measure choices in the same contest. This occurs when a source reports write-in votes alongside measure choices. The L1 parser handles each row independently — "For" is routed to `BallotMeasure`, while "Write-in" in the same contest is routed to `TurnoutMetadata`.

**Retention elections.** Judicial retention elections ask "Shall Judge X be retained?" with choices "Yes" and "No." These are structurally ballot measures but semantically candidate races — the "candidate" is the judge. L1 classifies these as `BallotMeasure` with an additional `retention_candidate` field preserving the judge's name from the contest string. This is an area where the boundary between candidate races and ballot measures is genuinely blurred.

## Scale

Ballot measure records account for approximately 3–5% of total rows in MEDSL 2022, varying by state. States with frequent ballot initiatives (California, Oregon, Colorado) have higher proportions. Failing to detect them does not just create bad entities — it inflates the count of "candidates" and distorts competitiveness metrics. A bond referendum with 51% "For" and 49% "Against" is not an uncontested race with one candidate named "For."