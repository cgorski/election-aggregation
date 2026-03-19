# Type System Design

The Rust type system enforces pipeline invariants at compile time. Records from different layers are different types. Contest kinds are an enum, not a string. Candidate names are a struct, not a `String`. Source-specific raw fields are typed per source. These choices eliminate categories of bugs that would otherwise surface at runtime — or worse, silently corrupt output.

## Layer-Typed Records

Each pipeline layer has its own record type. You cannot pass an L1 record to a function that expects L2, or accidentally mix L3 and L4 records in the same collection.

```
pub struct L0Record {
    pub raw_bytes: PathBuf,
    pub manifest: AcquisitionManifest,
}

pub struct L1Record {
    pub election: Election,
    pub jurisdiction: Jurisdiction,
    pub contest: Contest,
    pub results: Vec<CandidateResult>,
    pub turnout: Option<Turnout>,
    pub source: SourceMetadata,
    pub provenance: Provenance,
}

pub struct L2Record {
    pub l1: L1Record,
    pub candidate_name_embedding: Vec<f32>,
    pub contest_name_embedding: Vec<f32>,
    pub jurisdiction_embedding: Vec<f32>,
    pub embedding_model: String,
    pub embedding_version: String,
}

pub struct L3Record {
    pub l2: L2Record,
    pub candidate_cluster_id: ClusterId,
    pub contest_cluster_id: ClusterId,
    pub match_confidence: f64,
    pub match_method: MatchMethod,
}

pub struct L4Record {
    pub l3: L3Record,
    pub canonical_candidate_name: CandidateName,
    pub canonical_contest_name: String,
    pub temporal_chain_id: Option<ChainId>,
    pub verification_status: VerificationStatus,
}
```

Each layer wraps the previous layer's record. An L3Record contains an L2Record which contains an L1Record. This nesting means every L4 record carries the full history back to L1. The compiler enforces that you cannot construct an L3Record without first having an L2Record — you cannot skip layers.

### What the compiler prevents

- **Mixing layers in a collection.** `Vec<L1Record>` and `Vec<L2Record>` are different types. A function that processes L2 records cannot accidentally receive L1 records.
- **Accessing fields that don't exist yet.** An L1 record has no `candidate_cluster_id`. Attempting to access it is a compile error, not a null pointer or missing key at runtime.
- **Skipping pipeline stages.** You cannot construct an L3Record without providing an L2Record. The type system encodes the dependency chain.

## ContestKind Enum

The `ContestKind` enum separates three fundamentally different record types that sources mix together in the same file.

```
pub enum ContestKind {
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

### What the compiler prevents

- **Treating "For" as a person name.** The `BallotMeasure` variant has `choices: Vec<BallotChoice>`, not `results: Vec<CandidateResult>`. A `BallotChoice` has a `choice_text: String` field, not a `CandidateName` struct. There is no code path where "For" enters the name decomposition logic.
- **Embedding turnout metadata.** L2 pattern-matches on `ContestKind` and only computes embeddings for `CandidateRace` variants. `TurnoutMetadata` records pass through without embedding. This is enforced by the match arms — the compiler requires all three variants to be handled.
- **Mixing candidate results with ballot choices.** You cannot push a `BallotChoice` into a `Vec<CandidateResult>`. They are different types.

## CandidateName Struct

Candidate names are a struct with seven fields, not a `String`. This is documented in detail in [Candidate Name Components](../schema/name-components.md). The Rust definition:

```
pub struct CandidateName {
    pub raw: String,
    pub first: Option<String>,
    pub middle: Option<String>,
    pub last: Option<String>,
    pub suffix: Option<String>,
    pub nickname: Option<String>,
    pub canonical_first: Option<String>,
}
```

### What the compiler prevents

- **Passing a raw name string where a parsed name is expected.** Functions that perform entity resolution take `&CandidateName`, not `&str`. You cannot call them with the raw string — you must parse first.
- **Forgetting to preserve the raw name.** The `raw` field is a required `String`, not `Option<String>`. Every `CandidateName` carries the original source text.
- **Confusing nickname with first name.** They are separate fields. Code that constructs a composite embedding string uses `canonical_first`, `middle`, `last`, and `suffix` — never `raw`, never `nickname` on its own.

## SourceRawFields Enum

Every L1 record preserves the original source columns in a typed enum. Each source has its own variant with its own struct.

```
pub enum SourceRawFields {
    Medsl(MedslRawRecord),
    Ncsbe(NcsbeRawRecord),
    OpenElections(OpenElectionsRawRecord),
    Vest(VestRawRecord),
    Clarity(ClarityRawRecord),
    Fec(FecRawRecord),
    Census(CensusRawRecord),
}

pub struct MedslRawRecord {
    pub year: i32,
    pub state: String,
    pub state_po: String,
    pub state_fips: String,
    pub state_cen: String,
    pub state_ic: String,
    pub office: String,
    pub county_name: String,
    pub county_fips: String,
    pub jurisdiction_name: String,
    pub jurisdiction_fips: String,
    pub candidate: String,
    pub district: String,
    pub dataverse: String,
    pub stage: String,
    pub special: String,
    pub writein: String,
    pub mode: String,
    pub totalvotes: String,
    pub candidatevotes: String,
    pub version: String,
    pub readme_check: String,
    pub magnitude: Option<i32>,
    pub party_detailed: String,
    pub party_simplified: String,
}

pub struct NcsbeRawRecord {
    pub county: String,
    pub election_date: String,
    pub precinct_code: String,
    pub precinct_name: String,
    pub contest_group_id: String,
    pub contest_type: String,
    pub contest_name: String,
    pub choice: String,
    pub choice_party: String,
    pub vote_for: i32,
    pub election_day: i64,
    pub one_stop: i64,
    pub absentee_by_mail: i64,
    pub provisional: i64,
    pub total_votes: i64,
}
```

### What the compiler prevents

- **Accessing a field that doesn't exist for a source.** MEDSL has no `vote_for` column. NC SBE has no `dataverse` column. The struct types enforce this. If you have a `NcsbeRawRecord`, you can access `vote_for`. If you have a `MedslRawRecord`, you cannot — the field does not exist on the type.
- **Losing source-specific fields during normalization.** The `SourceRawFields` enum is a required field on `SourceMetadata`. The compiler forces every parser to populate it. No source's original columns are silently dropped.
- **Confusing source schemas.** Pattern matching on `SourceRawFields` requires handling each variant. Code that needs MEDSL-specific logic matches on `SourceRawFields::Medsl(ref raw)` and gets a `MedslRawRecord` with the correct field types.

## Other Type-Level Guarantees

**ClusterId and ChainId are newtypes, not raw strings.** They wrap a `String` but are distinct types. You cannot accidentally pass a `ClusterId` where a `ChainId` is expected.

```
pub struct ClusterId(pub String);
pub struct ChainId(pub String);
```

**Confidence, MatchMethod, and VerificationStatus are enums, not strings.** The set of valid values is fixed at compile time.

```
pub enum Confidence { High, Medium, Low }
pub enum MatchMethod { Deterministic, Embedding, LlmConfirmed }
pub enum VerificationStatus { MultiSourceConfirmed, LlmConfirmed, SingleSourceUnverified }
```

**Vote counts are `u64`, not `String`.** Source files sometimes contain non-integer vote values (0.1% of MEDSL 2022). These are caught during L1 parsing and quarantined — they never enter the typed record as a string that downstream code must re-parse.

## Design Tradeoffs

**Nesting vs. flattening.** L4Record contains L3Record contains L2Record contains L1Record. This means an L4 record is large — it carries the full history. The alternative (separate storage with ID references) would reduce memory per record but require joins to reconstruct provenance. We chose nesting because provenance integrity is a core requirement: every L4 record must be independently verifiable without external lookups.

**Per-source structs vs. generic key-value map.** Storing raw fields as `HashMap<String, String>` would be simpler to implement and would handle any source without code changes. We chose per-source structs because the fields are known at development time, and type safety catches schema drift (a renamed column breaks compilation, not data). The cost is that adding a new source requires defining a new struct and a new enum variant.

**Option fields vs. separate types per completeness level.** Many fields are `Option<String>` because not all sources provide them. An alternative design would define separate types for "fully populated" and "partially populated" records. We chose `Option` because the partially-populated case is the norm, not the exception — fewer than 5% of records have turnout data, and zero records have all fields populated.