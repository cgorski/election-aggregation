# Why This Is Hard

US local election results are published by approximately 3,143 county-level election offices, 50 state election boards, and an unknown number of municipal clerks. There is no common schema, no shared identifier system, and no central repository. This chapter describes the five structural problems that make unification difficult.

## Fragmented administration

US elections are administered at the county level. Each county decides independently how to collect, tabulate, and publish results. Some publish precinct-level CSV files. Others post scanned PDFs. Some use Clarity/Scytl election night reporting platforms that expose structured XML. Others put results on pages that require JavaScript rendering.

There is no federal mandate requiring any particular publication format. The result is 3,143 independent data silos with different schemas, different update schedules, different URL structures, and different retention policies.

When we downloaded precinct-level data for all 50 states from the MIT Election Data + Science Lab (MEDSL), we received 51 separate files containing 12.3 million rows. The files use different column encodings, different candidate name conventions, and different definitions of what constitutes a "local" race. Seven states — California, Iowa, Kansas, New Jersey, Pennsylvania, Tennessee, and Wisconsin — had zero local race records in the MEDSL 2022 dataset.

## No standard schema

The same vote record looks different in every source. Here is a single result — Shannon W. Bray in a 2022 North Carolina precinct — as represented by three different sources:

**MEDSL** (25-column CSV, one row per vote mode):
```
precinct,office,party_simplified,mode,votes,candidate,...
12-13,US SENATE,LIBERTARIAN,ELECTION DAY,47,SHANNON W BRAY,...
```

**NC SBE** (15-column TSV, vote modes as columns):
```
County	Precinct	Contest Name	Choice	Election Day	One Stop	Absentee by Mail	Provisional	Total Votes
CABARRUS	12-13	US SENATE	Shannon W. Bray	47	38	5	0	90
```

**OpenElections** (7-column CSV, totals only):
```
county,precinct,office,party,candidate,votes
Cabarrus,12-13,U.S. Senate,LIB,Shannon W. Bray,90
```

Three sources. Three schemas. Three representations of the candidate name (`SHANNON W BRAY`, `Shannon W. Bray`, `Shannon W. Bray`). Three levels of granularity for vote mode data. Three different office name formats (`US SENATE`, `US SENATE`, `U.S. Senate`). This is a federal race where all three sources agree on the totals. For local races, the divergence is worse.

## Name formatting differs across every source

We compared MEDSL and NC SBE data for 640 contests in the 2022 North Carolina general election where both sources reported the same vote totals. In 401 of those contests (63%), candidate names are formatted differently between the two sources.

The differences are systematic:

| Pattern | MEDSL | NC SBE |
|---------|-------|--------|
| Case | `SHANNON W BRAY` | `Shannon W. Bray` |
| Middle initial punctuation | `VICTORIA P PORTER` | `Victoria P. Porter` |
| Nickname quoting | `MICHAEL "STEVE" HUBER` | `Michael (Steve) Huber` |
| Suffix formatting | `ROBERT VAN FLETCHER JR` | `Robert Van Fletcher, Jr.` |
| Nickname style | `LM "MICKEY" SIMMONS` | `L.M. (Mickey) Simmons` |
| Write-in label | `WRITEIN` | `Write-In (Miscellaneous)` |

Each source applies a consistent internal convention. MEDSL uses ALL CAPS with no punctuation. NC SBE uses Title Case with periods and commas. Across sources, the conventions diverge.

The formatting problem is solvable with normalization rules. The deeper problem is name *identity*. We tested real candidate pairs with OpenAI's `text-embedding-3-large` model (3,072 dimensions):

| Name A | Name B | Cosine similarity | Same person? |
|--------|--------|:-----------------:|:------------:|
| `Charlie Crist` | `CRIST, CHARLES JOSEPH` | 0.451 | Yes |
| `Robert Williams` | `Robert Williams Jr.` | 0.862 | No |
| `Nikki Fried` | `Nicole Fried` | 0.642 | Yes |
| `Ron DeSantis` | `DESANTIS, RON` | 0.729 | Yes |

"Charlie Crist" and "CRIST, CHARLES JOSEPH" score 0.451 — below any reasonable match threshold — because `Charlie` and `Charles` have unrelated vector representations. These are the same person (same state, same office, identical vote count of 3,101,652). Only a language model with knowledge that Charlie is a common nickname for Charles can make the connection.

"Robert Williams" and "Robert Williams Jr." score 0.862 — above most auto-accept thresholds used in the literature. These are different people. The "Jr." suffix indicates a generational distinction. A system that auto-accepts at 0.82 would merge a father and son into one entity.

## Institutional variation by state

Across all 50 states in the 2022 MEDSL data, we found 8,387 unique local office names. Our keyword classifier handles 62% of them. The remaining 38% includes offices where the same title has different institutional meanings depending on the state.

**"County Judge"** in Texas is the presiding officer of the Commissioners Court — the chief executive of the county, analogous to a county manager. In every other state, a county judge presides over a courtroom. Texas has 254 counties; each has a County Judge who is an executive, not a judicial officer.

**"Sheriff"** in Connecticut is a court officer who serves civil process. In the other 49 states, the sheriff runs the county jail and patrols unincorporated areas.

**"Board of Education"** is an elected body in some states and an appointed body in others. Where it is appointed, it does not appear in election data — its absence from a source does not mean the county lacks a school board.

A static lookup table mapping office names to categories does not work. The classification must account for state-level context, which is why the pipeline uses a four-tier classifier: keyword matching for unambiguous names, regex patterns for structured names, embedding similarity against a reference set, and a language model for genuinely ambiguous cases.

## No persistent candidate identifiers

Timothy Lance won a seat on the Columbus County Schools Board of Education in 2022. No existing dataset can answer whether he ran before, whether he won, or whether the "T. Lance" who appeared on a 2018 ballot is the same person.

MEDSL, NC SBE, and OpenElections each treat every election as an independent snapshot. There is no identifier linking `Timothy Lance (2022)` to `Timothy Lance (2020)` to `Tim Lance (2018)`. The candidate name can change between elections — a middle initial added, a nickname used, a suffix dropped after a parent's death. The office can change if the candidate runs for a different seat. The county can change if the candidate relocates.

In 10 years of NC SBE data (2014–2024), we found 702 candidates appearing in three or more election cycles using exact name matching within the same county. George Dunlap appeared on the Mecklenburg County ballot in six consecutive cycles. Paul Beaumont in Currituck County ran for the Board of Commissioners, then the Board of Education, then back to Commissioners.

Connecting these records — determining that entries in different elections, from different sources, with different formatting, refer to the same person — requires preserving every name component through the cleaning pipeline, embedding candidates for vector retrieval, and confirming ambiguous matches with a language model that reasons about context (office, county, party, vote totals). This process is called entity resolution, and it is detailed in [its own chapter](../hard-problems/entity-resolution.md).

## What this adds up to

In 2022, across the MEDSL data for all 50 states, 48.8% of classified local races had only one candidate. In Minnesota, the uncontested rate was 89.3%. Nineteen local races ended in exact ties. Forty-three were decided by a single vote. These are basic facts about American democracy that require combining data from multiple sources, resolving thousands of name variations, classifying thousands of office types, and linking candidates across elections.

That is what this project does. The rest of this book describes how.