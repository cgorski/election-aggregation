# Schema Overview

The unified schema defines the structure of every election record at every pipeline layer. A single record represents one candidate's (or one ballot measure choice's) vote count in one geographic unit for one contest. All sources — MEDSL, NC SBE, OpenElections, VEST, Clarity — are normalized into this schema at L1. Subsequent layers (L2–L4) add fields but never remove them.

A record has six sections: **election**, **jurisdiction**, **contest**, **results**, **turnout**, **source**, and **provenance**. Not every field is populated for every record. Fields that the source does not provide are null, not inferred.

---

## Election

Identifies which election this record belongs to.

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `date` | date | Election date (ISO 8601) | `2022-11-08` |
| `year` | integer | Election year, derived from `date` | `2022` |
| `type` | ElectionType | General, primary, runoff, special, etc. | `General` |
| `stage` | string | Source-provided stage code | `GEN` |
| `special` | boolean | Whether this is a special election | `false` |
| `certification_status` | string | Certified, unofficial, or unknown | `certified` |

The `type` field is an enum — see [Enumerations Reference](./enumerations.md). The `stage` field preserves the raw source value (MEDSL uses `GEN`/`PRI`/`RUN`; NC SBE does not have a stage column). The `certification_status` field reflects whether the source data represents certified results. NC SBE and MEDSL publish certified data. Clarity publishes unofficial election night results that may be updated.

---

## Jurisdiction

Identifies the geographic unit where votes were counted.

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `state` | string | Full state name | `North Carolina` |
| `state_po` | string | Two-letter postal code | `NC` |
| `state_fips` | string | Two-digit state FIPS code | `37` |
| `county` | string | County name (may be null for statewide) | `Wake` |
| `county_fips` | string | Five-digit county FIPS code | `37183` |
| `precinct` | string | Precinct name or code from the source | `01-01` |
| `precinct_code` | string | Numeric precinct code (NC SBE only) | `0101` |
| `jurisdiction_name` | string | Jurisdiction name from MEDSL | `WAKE` |
| `jurisdiction_fips` | string | Jurisdiction FIPS from MEDSL | `37183` |
| `ocd_id` | string | Open Civic Data identifier (when available) | `ocd-division/country:us/state:nc/county:wake` |
| `level` | JurisdictionLevel | Geographic granularity of this record | `Precinct` |

The `county_fips` field is the primary geographic join key across sources. It is enriched from Census FIPS reference files at L1 when the source provides a county name but no code. The `ocd_id` field is populated when a mapping exists; it is null for most records today.

The `level` field indicates what geographic unit this row represents. Most records are `Precinct`. Some sources provide only county-level aggregates (`County`). VEST data with precinct boundaries is `Precinct` with accompanying geometry.

---

## Contest

Describes the race or ballot measure.

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `kind` | ContestKind | `CandidateRace`, `BallotMeasure`, or `TurnoutMetadata` | `CandidateRace` |
| `raw_name` | string | Contest name exactly as it appears in the source | `CABARRUS COUNTY SCHOOLS BOARD OF EDUCATION` |
| `normalized_name` | string | Cleaned contest name (L1+) | `Cabarrus County Schools Board of Education` |
| `office_level` | OfficeLevel | Federal, state, county, municipal, etc. | `County` |
| `office_category` | OfficeCategory | Executive, legislative, judicial, school board, etc. | `SchoolBoard` |
| `district` | string | District number or name (blank if at-large) | `DISTRICT 02` |
| `dataverse` | string | MEDSL's race level tag (blank for local) | `` |
| `classifier_method` | ClassifierMethod | How office_level and office_category were assigned | `Keyword` |
| `vote_for` | integer | Maximum number of candidates a voter may select | `1` |
| `magnitude` | integer | Number of seats being filled | `3` |
| `is_retention` | boolean | Whether this is a judicial retention election | `false` |

The `kind` field is an enum with three variants — see [Contest Kinds](./contest-kinds.md). The distinction between `CandidateRace`, `BallotMeasure`, and `TurnoutMetadata` is determined at L1 based on the contest name and choice values.

The `classifier_method` field records how the `office_level` and `office_category` were assigned: `Keyword` (deterministic string match, 62% of records), `Regex` (pattern-based, ~15%), `Embedding` (nearest-neighbor at L2), or `Llm` (LLM classification at L3). This field exists so that users can filter by classification confidence.

The `vote_for` field comes from NC SBE's `Vote For` column. MEDSL does not provide this field. When unavailable, it defaults to null. The `magnitude` field comes from MEDSL's `magnitude` column and indicates multi-member districts.

---

## Results

An array of candidate results attached to the contest. For a `CandidateRace`, each element is one candidate. For a `BallotMeasure`, each element is one choice (e.g., "For", "Against"). For `TurnoutMetadata`, the results array is empty.

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `candidate_name` | CandidateName | Decomposed name — see below | (see Name Components) |
| `party_raw` | string | Party label exactly as source provides | `LIBERTARIAN` |
| `party_simplified` | PartySimplified | Normalized party enum | `Libertarian` |
| `votes_total` | integer | Total votes for this candidate in this precinct | `90` |
| `vote_share` | float | Fraction of total contest votes (computed) | `0.023` |
| `writein` | boolean | Whether this is a write-in candidate | `false` |
| `incumbent` | boolean | Whether this candidate is the incumbent (if known) | `null` |
| `vote_counts_by_type` | VoteCountsByType | Breakdown by vote method — see below | (see below) |

### CandidateName

Names are decomposed into components rather than stored as a single string. This is documented in detail in [Candidate Name Components](./name-components.md).

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `raw` | string | Name exactly as it appears in the source | `MICHAEL "STEVE" HUBER` |
| `first` | string | Parsed first name | `Michael` |
| `middle` | string | Parsed middle name or initial | `null` |
| `last` | string | Parsed last name | `Huber` |
| `suffix` | string | Jr, Sr, II, III, IV, etc. | `null` |
| `nickname` | string | Detected nickname | `Steve` |
| `canonical_first` | string | Nickname-resolved first name | `Stephen` |

The `raw` field is preserved at every layer and never modified. The component fields are populated at L1 during name parsing. The `canonical_first` field is populated at L1 using the nickname dictionary (e.g., Charlie→Charles, Steve→Stephen, Pat→Patricia). All fields are available at every pipeline layer.

### VoteCountsByType

When the source provides vote mode breakdowns, they are stored here. NC SBE provides all four fields for every contest. MEDSL provides them when modes are split into separate rows (summed during L1). Most other sources provide only the total.

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `election_day` | integer | Election day votes | `136` |
| `early` | integer | Early / one-stop votes | `159` |
| `absentee_mail` | integer | Mail-in absentee votes | `7` |
| `provisional` | integer | Provisional ballot votes | `1` |

NC SBE calls early voting "One Stop." MEDSL calls it "EARLY VOTING." Both are mapped to the `early` field at L1.

---

## Turnout

Voter registration and participation counts for the geographic unit. These fields are sparsely populated — less than 5% of records have values.

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `registered_voters` | integer | Number of registered voters in this precinct | `2847` |
| `ballots_cast` | integer | Total ballots cast in this precinct | `1893` |
| `turnout_pct` | float | `ballots_cast / registered_voters` (computed) | `0.665` |

NC SBE provides `registered_voters` via "Registered Voters" pseudo-contest rows. These are extracted during L1 parsing and attached to the precinct's turnout object. MEDSL rarely includes registration counts. Most records have null turnout.

---

## Source

Provenance fields that document where this record came from.

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `source_type` | SourceType | Enum identifying the source system | `Medsl` |
| `source_file` | string | Filename of the L0 artifact | `2022-nc-local-precinct-general.csv` |
| `source_row` | integer | Row number in the source file | `14523` |
| `retrieval_date` | datetime | When the source file was downloaded (UTC) | `2025-01-15T03:22:00Z` |
| `confidence` | Confidence | `High`, `Medium`, or `Low` | `Medium` |
| `raw_fields` | SourceRawFields | All original columns from the source, typed per source | (see below) |

### SourceRawFields

The `raw_fields` object preserves every column from the original source row, typed as an enum per source. This ensures no information is lost during normalization.

| Variant | Source | Fields preserved |
|---------|--------|-----------------|
| `MedslRawRecord` | MEDSL | All 25 MEDSL columns including `state_cen`, `state_ic`, `readme_check`, `version` |
| `NcsbeRawRecord` | NC SBE | All 15 NC SBE columns including `Contest Group ID`, `Contest Type`, `Real Precinct` |
| `OpenElectionsRawRecord` | OpenElections | Variable columns depending on state file |
| `VestRawRecord` | VEST | Encoded column names and geometry reference |
| `ClarityRawRecord` | Clarity | XML element attributes |
| `FecRawRecord` | FEC | All 15 `cn.txt` columns |
| `CensusRawRecord` | Census | FIPS file columns |

Each variant is a struct with typed fields matching the source schema. This is a Rust enum, not a JSON object — the type system ensures you cannot accidentally read an NC SBE field from a MEDSL record. See [Type System Design](../rust/type-system.md).

---

## Provenance

Hash chain and version metadata that enable verification and reproducibility.

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `record_id` | string | Deterministic hash of (source, file, row) | `a3f8c2...` |
| `l1_hash` | string | SHA-256 hash of this L1 record's content | `7b2e91...` |
| `l0_parent_hash` | string | SHA-256 hash of the L0 source artifact | `c4d1f0...` |
| `l0_byte_offset` | integer | Byte offset in the L0 file where this row starts | `1048576` |
| `parser_version` | string | Version of the parser that produced this record | `0.1.0` |
| `schema_version` | string | Version of the schema this record conforms to | `1.0.0` |

The hash chain links every record back to the original source bytes. If the L1 record is modified, its `l1_hash` changes and no longer matches the hash stored in any L2 record that references it. The verification algorithm at L4 checks the full chain: L4 → L3 → L2 → L1 → L0 → source bytes.

The `record_id` is deterministic: identical source input always produces the same `record_id`. This enables deduplication and makes re-processing idempotent.

---

## Layer-Specific Additions

Each pipeline layer adds fields to the record. The base schema (above) is fully populated at L1. Subsequent layers extend it:

| Layer | Fields added |
|-------|-------------|
| L2 (Embedded) | `candidate_name_embedding`, `contest_name_embedding`, `jurisdiction_embedding`, `embedding_model`, `embedding_version` |
| L3 (Matched) | `candidate_cluster_id`, `contest_cluster_id`, `match_confidence`, `match_method` |
| L4 (Canonical) | `canonical_candidate_name`, `canonical_contest_name`, `temporal_chain_id`, `verification_status`, `alias_table` |

L1 records are self-contained. L2+ records reference their parent layer's hash. No fields from earlier layers are removed or overwritten — each layer is additive.

---

## JSONL Representation

At every layer, records are serialized as one JSON object per line (JSONL). The six sections are top-level keys:

```
{"election":{"date":"2022-11-08","year":2022,"type":"General",...},"jurisdiction":{"state":"North Carolina","state_po":"NC",...},"contest":{"kind":"CandidateRace","raw_name":"CABARRUS COUNTY SCHOOLS BOARD OF EDUCATION",...},"results":[{"candidate_name":{"raw":"GREG MILLS","first":"Greg","last":"Mills",...},"votes_total":79,...}],"turnout":null,"source":{"source_type":"Medsl","source_file":"2022-nc-local-precinct-general.csv",...},"provenance":{"record_id":"a3f8c2...","l1_hash":"7b2e91...",...}}
```

Files are streamable: each line is a complete record. Files are appendable: new records can be concatenated without modifying existing lines. Serialization uses `serde_json` in Rust. See [Output Formats](../rust/output-formats.md).