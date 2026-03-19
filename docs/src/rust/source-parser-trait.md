# The SourceParser Trait

Every data source in the pipeline implements a single trait: `SourceParser`. This trait defines the contract between source-specific parsing logic and the generic pipeline infrastructure. Adding a new source means implementing one trait.

## Trait definition

```rust
pub trait SourceParser {
    /// The raw record type specific to this source.
    type RawRecord;

    /// Parse the source file into an iterator of raw records.
    ///
    /// This reads bytes from L0 and produces typed records that
    /// preserve every column from the source. No normalization
    /// occurs here — just deserialization.
    fn parse(&self, l0_bytes: &[u8]) -> Box<dyn Iterator<Item = Result<Self::RawRecord, ParseError>>>;

    /// Convert a single raw record into an L1 record.
    ///
    /// This is where normalization happens: name decomposition,
    /// party normalization, FIPS enrichment, contest kind
    /// classification, and hash computation.
    fn to_l1(&self, raw: Self::RawRecord) -> Result<L1Record, TransformError>;

    /// Source metadata for provenance tracking.
    fn source_type(&self) -> SourceType;
}
```

The trait is generic over `RawRecord`. Each source defines its own raw record struct matching the source schema column-for-column. MEDSL has a 25-field `MedslRawRecord`. NC SBE has a 15-field `NcsbeRawRecord`. This prevents cross-source field access at compile time.

## How the pipeline uses the trait

The pipeline is generic over `SourceParser`. Each layer invokes the trait methods without knowing which source it is processing:

```rust
fn process_l0_to_l1<S: SourceParser>(
    source: &S,
    l0_artifact: &L0Artifact,
) -> impl Iterator<Item = Result<L1Record, PipelineError>> {
    let raw_records = source.parse(&l0_artifact.bytes);

    raw_records.map(move |raw_result| {
        let raw = raw_result?;
        let l1 = source.to_l1(raw)?;
        Ok(l1)
    })
}
```

Records are processed one at a time as an iterator. The full file is never loaded into memory as a collection of parsed records. This enables processing multi-gigabyte source files (MEDSL's 2020 dataset is 13.2M rows) with bounded memory.

## NC SBE implementation sketch

The NC SBE source illustrates what a concrete implementation looks like. NC SBE files are tab-delimited with 15 columns (2014–2024 schema).

The raw record preserves all source columns:

```rust
pub struct NcsbeRawRecord {
    pub county: String,
    pub election_date: String,
    pub precinct_code: String,
    pub precinct_name: String,
    pub contest_group_id: String,
    pub contest_type: String,        // "S" = statewide, "C" = county/local
    pub contest_name: String,
    pub choice: String,
    pub choice_party: String,
    pub vote_for: u32,
    pub election_day: u64,
    pub one_stop: u64,
    pub absentee_by_mail: u64,
    pub provisional: u64,
    pub total_votes: u64,
}
```

The `parse` method handles tab splitting and type conversion:

```rust
impl SourceParser for NcsbeSource {
    type RawRecord = NcsbeRawRecord;

    fn parse(&self, l0_bytes: &[u8]) -> Box<dyn Iterator<Item = Result<NcsbeRawRecord, ParseError>>> {
        let reader = BufReader::new(l0_bytes);
        Box::new(reader.lines().skip(1).map(|line| {
            let line = line?;
            let fields: Vec<&str> = line.split('\t').collect();
            // ... field extraction and type conversion
            Ok(NcsbeRawRecord { /* ... */ })
        }))
    }

    fn to_l1(&self, raw: NcsbeRawRecord) -> Result<L1Record, TransformError> {
        // 1. Classify contest kind
        let kind = classify_contest(&raw.contest_name, &raw.choice);

        // 2. Decompose candidate name
        let name = decompose_name_ncsbe(&raw.choice);

        // 3. Build vote counts from the four mode columns
        let vote_counts = VoteCountsByType {
            election_day: Some(raw.election_day),
            early: Some(raw.one_stop),
            absentee_mail: Some(raw.absentee_by_mail),
            provisional: Some(raw.provisional),
        };

        // 4. Determine office level from Contest Type
        let office_level = match raw.contest_type.as_str() {
            "S" => classify_statewide_office(&raw.contest_name),
            "C" => classify_local_office(&raw.contest_name),
            _   => OfficeLevel::Other,
        };

        // 5. Build provenance
        let l1_hash = compute_hash(&raw);

        Ok(L1Record { /* ... */ })
    }

    fn source_type(&self) -> SourceType {
        SourceType::Ncsbe2022
    }
}
```

Key points in the NC SBE `to_l1` implementation:

- **Vote mode columns map directly.** NC SBE is the only source where all four mode fields (`election_day`, `one_stop`, `absentee_by_mail`, `provisional`) are always present. No row-level aggregation is needed, unlike MEDSL where modes are separate rows.
- **`Contest Type` drives office classification.** The `C`/`S` flag tells us immediately whether a race is local or statewide, reducing the keyword classifier's job.
- **Name decomposition uses NC SBE conventions.** Nicknames are in parentheses (not double quotes as in MEDSL). Suffixes follow commas. The parser for NC SBE and the parser for MEDSL call different name-parsing functions.

## Adding a new source

To add a new source (e.g., a state portal for Ohio):

1. Define `OhioRawRecord` with fields matching the source schema.
2. Implement `SourceParser` for `OhioSource`.
3. Write `parse` to handle the source format (CSV, TSV, XML, JSON).
4. Write `to_l1` to normalize names, classify contests, enrich FIPS codes, and compute hashes.
5. Add the source to the `SourceType` enum.

The pipeline infrastructure — streaming, partitioning, JSONL serialization, hash chaining — is reused without modification. The only new code is the source-specific parsing and normalization logic in the trait implementation.

## Error handling

Both `parse` and `to_l1` return `Result`. Errors are not fatal. A row that fails to parse (malformed TSV, non-integer vote count, encoding issue) produces an error that the pipeline routes to a quarantine log. Processing continues with the next row.

MEDSL's `votes` column contains 12,782 non-integer values out of 12.3M rows (0.1%) in 2022. These rows are quarantined at `parse` time, logged with the source file name and row number, and excluded from L1 output. The quarantine log is itself a JSONL file, enabling post-processing review.