# What Questions Should Be Answerable?

The purpose of this project is to make US local election data queryable. Across 42 million rows, 50 states, and 8,387 distinct office names, basic questions remain difficult to answer. This chapter frames those questions by audience.

## Four audiences, different needs

- **Journalists** need specific, verifiable facts — closest races, unopposed incumbents, anomalies worth investigating. See [For Journalists](./questions-journalists.md).
- **Researchers** need structured, reproducible datasets — uncontested rates by office type, candidate career paths, cross-state comparisons. See [For Researchers](./questions-researchers.md).
- **Government staffers** need operational inventories — what offices exist in a jurisdiction, how many races appear on a ballot, how local structures compare to peer counties. See [For Government Staffers](./questions-government.md).
- **Civic tech developers** need reliable data interchange — OCD-ID mappings, entity-resolved candidate records, JSONL exports for downstream applications. See [For Civic Tech Developers](./questions-civictech.md).

## What the data already tells us

Even partial analysis of available sources reveals findings that are difficult to obtain elsewhere:

- **48.8%** of local races in available data are uncontested — one candidate, no opponent.
- **19** exact ties have been identified across the dataset (same vote total, different candidates).
- **8,387** unique office name strings exist before normalization, many referring to the same underlying office.

These numbers are not estimates. They come from deterministic queries against cleaned, source-attributed JSONL records. The methodology for each finding is documented in the [recipe chapters](../usage/recipes.md).

## Why these questions matter

No single existing source answers all of these questions. The [existing landscape](./existing-landscape.md) chapter surveys what is available today and where each source falls short. This project exists to fill the gaps — not by replacing those sources, but by unifying them through a documented, reproducible pipeline.