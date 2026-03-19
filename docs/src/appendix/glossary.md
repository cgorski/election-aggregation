# Glossary

**Blocking.** A preprocessing step in entity resolution that partitions records into groups (blocks) that share a key attribute — typically state + office type or county FIPS code. Only records within the same block are compared, reducing the number of pairwise comparisons from O(n²) to a tractable subset.

**Composite string.** A concatenated text representation of a record used as input to an embedding model. A candidate composite string might combine name, office, jurisdiction, party, and year into a single string. The template that defines which fields are included and in what order is versioned and stored in the L2 manifest.

**Cosine similarity.** A measure of similarity between two vectors, computed as the cosine of the angle between them. Ranges from -1 to 1; values closer to 1 indicate higher similarity. Used at L3 to compare candidate and contest embeddings. The pipeline uses a threshold of 0.88 for embedding-based entity matches.

**Entity resolution.** The process of determining whether two records refer to the same real-world entity (person, office, or contest) despite differences in formatting, naming, or source. The pipeline uses a four-step cascade: exact match → Jaro-Winkler → embedding similarity → LLM confirmation.

**FAISS.** Facebook AI Similarity Search. A library for efficient similarity search over dense vector collections. Used at L3 to perform approximate nearest-neighbor lookups over L2 embeddings when comparing candidate records across sources.

**FIPS code.** Federal Information Processing Standards code. A numeric identifier assigned by the Census Bureau to states (2 digits), counties (5 digits: 2 state + 3 county), and other geographic entities. Example: `37119` = Mecklenburg County, North Carolina. Used as a join key across sources.

**Jaro-Winkler similarity.** A string similarity metric that gives higher scores to strings that match from the beginning. Ranges from 0 to 1. The pipeline uses a threshold of 0.92 for name matching. Preferred over edit distance for person names because prefix agreement is a strong signal of identity.

**JSONL.** JSON Lines. A text format where each line is a valid JSON object, separated by newlines. The pipeline uses JSONL as the storage and interchange format at every layer (L0–L4). One record per line enables streaming reads and line-level integrity checks.

**L0 (Raw).** The first pipeline layer. Byte-identical copies of source files as retrieved. No parsing, no transformation. Stored with retrieval timestamps and SHA-256 hashes.

**L1 (Cleaned).** The second layer. Deterministic parsing, field extraction, name normalization, and FIPS enrichment. Output is structured JSONL with a consistent schema regardless of source format.

**L2 (Embedded).** The third layer. Adds vector embeddings (text-embedding-3-large, 3072 dimensions) and office classification results. Deterministic given L1 input and a fixed model version.

**L3 (Matched).** The fourth layer. Entity resolution — linking records that refer to the same candidate, contest, or office across sources and years. Non-deterministic steps (LLM calls) are recorded in the decision log for replay.

**L4 (Canonical).** The fifth layer. Assigns canonical names, deduplicates records, selects authoritative values, and produces the final queryable dataset. Deterministic given L3 input.

**OCD-ID.** Open Civic Data Identifier. A hierarchical string identifier for political geographies, following the pattern `ocd-division/country:us/state:nc/county:mecklenburg`. Used to link jurisdictions across datasets that may use different naming conventions.

**Precinct.** The smallest administrative unit for election administration. Voters are assigned to a precinct based on their address. Precinct-level results, when available, provide the most granular view of voting patterns. Coverage varies — some sources report only county-level totals.