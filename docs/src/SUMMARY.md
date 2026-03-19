# Summary

[Introduction](./introduction.md)

---

# I. The Problem

- [Why This Is Hard](./problem/why-hard.md)
- [What Questions Should Be Answerable](./problem/questions.md)
    - [For Journalists](./problem/questions-journalists.md)
    - [For Researchers](./problem/questions-researchers.md)
    - [For Government Staffers](./problem/questions-government.md)
    - [For Civic Tech Developers](./problem/questions-civictech.md)
- [What Exists Today and Where It Falls Short](./problem/existing-landscape.md)

---

# II. Data Sources

- [Source Overview](./sources/overview.md)
- [Coverage Matrix](./sources/coverage-matrix.md)
- [What We Cover, What We Don't, and Why](./sources/honest-scope.md)
- [MEDSL — MIT Election Data + Science Lab](./sources/medsl.md)
- [NC SBE — North Carolina State Board of Elections](./sources/ncsbe.md)
- [OpenElections](./sources/openelections.md)
- [Clarity/Scytl ENR](./sources/clarity.md)
- [VEST — Voting and Election Science Team](./sources/vest.md)
- [Census Bureau FIPS Reference](./sources/census.md)
- [FEC Candidate Master Files](./sources/fec.md)
- [Future Sources](./sources/future.md)

---

# III. The Hard Problems

- [Name Normalization](./hard-problems/name-normalization.md)
    - [Nicknames and Middle Initials](./hard-problems/names-nicknames-and-middles.md)
    - [Suffixes: Jr/Sr Means Different People](./hard-problems/names-suffixes.md)
    - [The Nickname Dictionary](./hard-problems/names-dictionary.md)
- [Office Classification](./hard-problems/office-classification.md)
    - [The Four-Tier Classifier](./hard-problems/office-four-tiers.md)
- [Entity Resolution](./hard-problems/entity-resolution.md)
    - [The Cascade: Exact → Jaro-Winkler → Embedding → LLM](./hard-problems/entity-cascade.md)
    - [Real Test Cases from Real Data](./hard-problems/entity-test-cases.md)
    - [Threshold Calibration](./hard-problems/entity-thresholds.md)
- [Non-Candidate Records](./hard-problems/non-candidates.md)
    - [Registered Voters, Ballots Cast, Over/Under Votes](./hard-problems/non-candidates-metadata.md)
    - [Ballot Measure Choices: For/Against/Yes/No](./hard-problems/non-candidates-ballot-measures.md)
- [Contest Disambiguation](./hard-problems/contest-disambiguation.md)
- [Cross-Source Reconciliation](./hard-problems/cross-source.md)

---

# IV. Architecture

- [Design Principles](./architecture/principles.md)
- [The Five-Layer Pipeline](./architecture/pipeline-overview.md)
    - [L0: Raw — Byte-Identical Source Preservation](./architecture/l0-raw.md)
    - [L1: Cleaned — Deterministic Parsing and Enrichment](./architecture/l1-cleaned.md)
    - [L2: Embedded — Vector Generation and Classification](./architecture/l2-embedded.md)
    - [L3: Matched — Entity Resolution and LLM Confirmation](./architecture/l3-matched.md)
    - [L4: Canonical — Authoritative Names and Verification](./architecture/l4-canonical.md)
- [Why the Order Matters: Clean → Embed → Match → Canonicalize](./architecture/ordering.md)
- [Provenance and the Hash Chain](./architecture/provenance.md)
- [The Project Does Not Store Data](./architecture/no-data-storage.md)
- [Embedding Model: text-embedding-3-large](./architecture/embeddings-model.md)
- [Composite String Templates](./architecture/embeddings-composites.md)
- [When the LLM Gets Called (And When It Doesn't)](./architecture/llm-when.md)
- [Budget Is Not a Constraint — Speed and Reproducibility Are](./architecture/llm-budget.md)

---

# V. Unified Schema

- [Schema Overview](./schema/overview.md)
- [Contest Kinds: CandidateRace, BallotMeasure, TurnoutMetadata](./schema/contest-kinds.md)
- [Candidate Name Components](./schema/name-components.md)
- [Enumerations Reference](./schema/enumerations.md)

---

# VI. Rust Implementation

- [Crate Overview](./rust/overview.md)
- [Type System Design](./rust/type-system.md)
- [The SourceParser Trait](./rust/source-parser-trait.md)
- [Pipeline Execution](./rust/pipeline-execution.md)
- [Output Format: JSONL and CSV Export](./rust/output-formats.md)
- [CLI Reference](./rust/cli.md)

---

# VII. Using the Data

- [Getting Started](./usage/getting-started.md)
- [Download the Data](./usage/download.md)
- [Run the Pipeline](./usage/run-pipeline.md)
- [Querying JSONL Output](./usage/query-jsonl.md)
- [Recipes](./usage/recipes.md)
    - [Closest Races in America](./usage/recipe-closest-races.md)
    - [Uncontested Race Rate by State](./usage/recipe-uncontested.md)
    - [Sheriff Accountability: Who Runs Unopposed?](./usage/recipe-sheriffs.md)
    - [School Board Competitiveness](./usage/recipe-school-boards.md)
    - [Office Inventory for a County](./usage/recipe-office-inventory.md)
    - [Career Tracking Across Elections](./usage/recipe-career-tracking.md)
    - [Verify a Specific Result](./usage/recipe-verify-result.md)

---

# VIII. Trust and Reproducibility

- [The Two Audiences](./trust/two-audiences.md)
- [Confidence Levels](./trust/confidence.md)
- [Reporting Errors](./trust/reporting-errors.md)
- [Known Limitations](./trust/limitations.md)

---

# Appendices

- [Full Nickname Dictionary](./appendix/nickname-dictionary.md)
- [Office Classification Reference](./appendix/office-classification.md)
- [NIST SP 1500-100 Alignment](./appendix/nist-alignment.md)
- [Research References](./appendix/references.md)
- [Glossary](./appendix/glossary.md)
- [Changelog](./appendix/changelog.md)