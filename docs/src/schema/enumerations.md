# Enumerations Reference

Every categorical field in the schema is represented by a closed enumeration. This chapter lists all enum types, their values, and where each is used.

## ElectionType

Classifies the type of election event.

| Value | Description |
|-------|-------------|
| `General` | Regular general election (November even years) |
| `Primary` | Party primary election |
| `Runoff` | Runoff election following an inconclusive primary or general |
| `Special` | Special election to fill a vacancy |
| `SpecialPrimary` | Primary for a special election |
| `SpecialRunoff` | Runoff for a special election |
| `Municipal` | Municipal election (may be odd-year) |
| `Recall` | Recall election |
| `Retention` | Judicial retention election |
| `Other` | Election type not matching any above category |

Source mapping: MEDSL's `stage` column maps `GEN` → `General`, `PRI` → `Primary`, `RUN` → `Runoff`. The `special` boolean flag promotes any type to its `Special*` variant. NC SBE does not distinguish — all loaded files are general elections.

## JurisdictionLevel

The geographic level at which a result is reported.

| Value | Description |
|-------|-------------|
| `State` | Statewide aggregate |
| `County` | County-level result |
| `Precinct` | Precinct-level result |
| `CongressionalDistrict` | Congressional district aggregate |
| `StateLegislativeUpper` | State senate district aggregate |
| `StateLegislativeLower` | State house/assembly district aggregate |
| `Municipality` | City or town |
| `SchoolDistrict` | School district boundary |

Most records in the pipeline are `Precinct`. County and state aggregates appear in OpenElections data where precinct-level files are unavailable.

## OfficeLevel

The level of government an office belongs to.

| Value | Description |
|-------|-------------|
| `Federal` | President, US Senate, US House |
| `Statewide` | Governor, AG, SOS, state auditor, state treasurer |
| `StateLegislature` | State senate, state house/assembly |
| `County` | County commissioner, county clerk, coroner, sheriff |
| `Municipal` | Mayor, city council, town board |
| `Judicial` | All judicial offices (federal, state, county, municipal) |
| `SchoolBoard` | School board / board of education |
| `SpecialDistrict` | Soil and water, fire district, utility district, transit |
| `Township` | Township supervisor, township trustee |
| `Other` | Unclassifiable after all four classifier tiers |

Assigned by the four-tier classifier at L1 (keyword), L2 (embedding), and L3 (LLM). The `Other` rate is 0.56% on NC test data.

## OfficeCategory

Finer-grained classification within an office level. One office level maps to many categories.

| Value | Description |
|-------|-------------|
| `Executive` | President, governor, mayor, county executive |
| `Legislative` | US House, US Senate, state legislature, city council |
| `Judicial` | Judge, justice, magistrate |
| `LawEnforcement` | Sheriff, constable, marshal |
| `FiscalOfficer` | Treasurer, auditor, comptroller, tax collector |
| `Clerk` | County clerk, clerk of court, register of deeds |
| `Education` | School board, board of education, superintendent |
| `PublicWorks` | Soil and water, utility district, surveyor |
| `Regulatory` | Coroner, medical examiner, public service commission |
| `PartyOffice` | Precinct committee officer, party chair (when on ballot) |
| `Other` | Does not fit the above categories |

## BallotMeasureType

Classifies ballot measures by their legal mechanism.

| Value | Description |
|-------|-------------|
| `BondIssue` | Debt authorization (general obligation or revenue bond) |
| `LevyRenewal` | Property tax levy renewal |
| `LevyNew` | New property tax levy |
| `ConstitutionalAmendment` | State constitutional amendment |
| `CharterAmendment` | Municipal or county charter amendment |
| `Referendum` | Legislative referendum referred to voters |
| `Initiative` | Citizen-initiated ballot measure |
| `Recall` | Recall question for a specific officeholder |
| `Other` | Measure type not determinable from contest name |

## PartySimplified

Normalized party affiliation. Preserves the most common parties as distinct values; collapses minor parties.

| Value | Description |
|-------|-------------|
| `Democrat` | Democratic Party |
| `Republican` | Republican Party |
| `Libertarian` | Libertarian Party |
| `Green` | Green Party |
| `Independent` | Independent / no party affiliation |
| `Nonpartisan` | Nonpartisan contest (no party on ballot) |
| `WriteIn` | Write-in candidate (party unknown or not applicable) |
| `Other` | Any other party (Constitution, Working Families, Reform, etc.) |

Source mapping: MEDSL's `party_simplified` column maps directly. NC SBE's `Choice Party` codes: `DEM` → `Democrat`, `REP` → `Republican`, `LIB` → `Libertarian`, `GRE` → `Green`, `UNA` → `Independent`, blank → `Nonpartisan`. FEC codes: `DEM`, `REP`, `LIB`, `GRE`, `IND`, `NNE` → `Nonpartisan`.

## SourceType

Identifies the origin of a record. One value per data source file type.

| Value | Description |
|-------|-------------|
| `Medsl2018` | MEDSL 2018 precinct-level file |
| `Medsl2020` | MEDSL 2020 precinct-level file |
| `Medsl2022` | MEDSL 2022 precinct-level file |
| `Ncsbe2014` | NC SBE 2014 general (15-column schema) |
| `Ncsbe2016` | NC SBE 2016 general |
| `Ncsbe2018` | NC SBE 2018 general |
| `Ncsbe2020` | NC SBE 2020 general |
| `Ncsbe2022` | NC SBE 2022 general |
| `Ncsbe2024` | NC SBE 2024 general |
| `NcsbeLegacy` | NC SBE 2006–2012 (older schemas) |
| `OpenElections` | OpenElections CSV (any state) |
| `ClarityXml` | Clarity/Scytl ENR XML extract |
| `VestShapefile` | VEST precinct shapefile |
| `CensusFips` | Census Bureau FIPS reference file |
| `FecCandidate` | FEC candidate master file (`cn.txt`) |
| `Manual` | Manually entered or corrected record |

Each L1 record carries exactly one `SourceType`. When sources are merged at L3/L4, the provenance chain preserves the original `SourceType` for every contributing record.

## ExtractionMethod

How a field value was obtained from the source.

| Value | Description |
|-------|-------------|
| `Direct` | Value copied directly from a source column |
| `Parsed` | Value extracted by parsing a combined field (e.g., name decomposition) |
| `Derived` | Value computed from other fields (e.g., vote share from votes/total) |
| `Enriched` | Value added from a reference source (e.g., FIPS code from Census lookup) |
| `Inferred` | Value inferred by model (embedding similarity or LLM) |

## Confidence

The verification level assigned to a record at L4.

| Value | Criteria |
|-------|----------|
| `High` | Confirmed by two or more independent sources with matching vote totals |
| `Medium` | Single source, certified state data or academic curated source |
| `Low` | Single source, community curated or unverified; or match confidence below threshold |

Confidence is assigned per-record, not per-source. A record from MEDSL that is corroborated by NC SBE receives `High`. A record from MEDSL with no second source receives `Medium`. A record from OpenElections with schema inconsistencies receives `Low`.

## ClassifierMethod

Which tier of the office classifier produced the office level and category.

| Value | Description |
|-------|-------------|
| `Keyword` | Matched a keyword or keyword phrase (e.g., "SHERIFF" → `LawEnforcement`) |
| `Regex` | Matched a regex pattern (e.g., `DISTRICT \d+` for legislative districts) |
| `Embedding` | Classified by nearest-neighbor embedding similarity at L2 |
| `Llm` | Classified by LLM at L3 after embedding was ambiguous |

Records carry the method so downstream consumers can filter by classifier reliability. `Keyword` and `Regex` are deterministic and reproducible. `Embedding` and `Llm` depend on model versions.

## GeoMatchMethod

How a geographic identifier was resolved.

| Value | Description |
|-------|-------------|
| `FipsExact` | FIPS code present in source and matched Census reference exactly |
| `NameExact` | Geographic name matched Census reference exactly (case-insensitive) |
| `NameFuzzy` | Geographic name matched after fuzzy normalization (e.g., "ST. LOUIS" → "St. Louis") |
| `OcdLookup` | Matched via Open Civic Data identifier |
| `Unresolved` | Could not be matched to a canonical geographic entity |

Most MEDSL records resolve via `FipsExact` (the source provides `county_fips`). NC SBE records resolve via `NameExact` after uppercasing the county name. OpenElections records frequently require `NameFuzzy` due to inconsistent county name formatting.