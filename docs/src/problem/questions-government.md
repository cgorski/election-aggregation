# For Government Staffers

County clerks, election administrators, and local officials need operational data — not research datasets. These are the questions government staff ask, with answers drawn from the dataset.

## Office inventories

- **What offices exist in my county?** Columbus County, NC has 25 distinct elected offices across county government, municipalities, school boards, and special districts. The pipeline produces per-county office inventories from L4 canonical records. See [Office Inventory for a County](../usage/recipe-office-inventory.md).
- **How many races will be on the next ballot?** Historical office inventories establish the set of offices that typically appear in a given election cycle. Odd-year vs. even-year patterns, staggered terms, and special elections are identifiable where source data includes election dates and term lengths.
- **Which offices are partisan vs. nonpartisan?** Party affiliation is recorded where the source provides it. In North Carolina, all county commissioner races are partisan; all school board races are nonpartisan. Coverage varies by state.

## Comparisons

- **How does our uncontested rate compare to peer counties?** County-level uncontested rates are computable for any jurisdiction with coverage. A county clerk can compare their 60% uncontested rate against the state median or against demographically similar counties. See [Uncontested Race Rate by State](../usage/recipe-uncontested.md).
- **Are other counties consolidating offices we still elect separately?** Office inventories across counties within a state reveal structural differences — some counties elect a coroner, others appoint one. The data does not explain *why*, but it shows *where* differences exist.
- **How many candidates typically file for each office?** Candidate counts per contest are derivable from L4 records. A county with historically 1.2 candidates per school board seat has a different recruitment problem than one averaging 3.4.

## Administrative planning

- **What does our ballot complexity look like over time?** The number of contests per jurisdiction per cycle is queryable. Ballot length affects printing costs, voter fatigue research, and polling place logistics.
- **Which districts overlap our jurisdiction?** Where OCD-IDs are present, hierarchical district relationships can be inferred. A county contains municipalities, school districts, and special districts — the data reflects which contests appear in which jurisdictions.

## Data format

All outputs are JSONL with one record per contest-candidate pair. Government staff who need spreadsheets can convert JSONL to CSV with standard tools. See [Querying JSONL Output](../usage/query-jsonl.md).

## Caveats

- Office inventories are only as complete as the source data. If a state does not report local results to MEDSL or another covered source, those offices will not appear.
- The pipeline documents sources and provides tools — it does not store or redistribute official election results. See [The Project Does Not Store Data](../architecture/no-data-storage.md).
- Seven states have zero local coverage in MEDSL 2022. Check the [Coverage Matrix](../sources/coverage-matrix.md) before relying on completeness for a specific jurisdiction.