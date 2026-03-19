# Full Nickname Dictionary

The pipeline applies nickname normalization at L1 to improve entity resolution at L3. When a candidate's first name matches a known nickname, the canonical form is stored in `canonical_first` and the original is preserved in `first`.

This dictionary is applied deterministically. Every name is checked against the table below. No context or heuristics are used — if the input matches the nickname column, the canonical column is applied. This means the mapping is fast and reproducible but occasionally wrong (see [The Ted Problem](#the-ted-problem) below).

## Mappings

| Nickname | Canonical | Notes |
|---|---|---|
| al | albert | |
| alex | alexander | |
| andy | andrew | |
| barb | barbara | |
| ben | benjamin | |
| bernie | bernard | |
| bert | albert | Also Herbert; resolved to albert by frequency |
| beth | elizabeth | |
| bill | william | |
| billy | william | |
| bob | robert | |
| bobby | robert | |
| bonnie | bonita | |
| bud | william | Regional; less reliable |
| charlie | charles | |
| chris | christopher | Also Christine; gendered ambiguity |
| chuck | charles | |
| cindy | cynthia | |
| dan | daniel | |
| danny | daniel | |
| dave | david | |
| deb | deborah | |
| debbie | deborah | |
| dick | richard | |
| don | donald | |
| doug | douglas | |
| drew | andrew | |
| ed | edward | |
| eddie | edward | |
| frank | franklin | Also Francis; resolved to franklin by frequency |
| fred | frederick | |
| gene | eugene | |
| gerry | gerald | |
| hank | henry | |
| harry | harold | Also Henry (British tradition); resolved to harold |
| jack | john | |
| jake | jacob | |
| jan | janice | Also Janet; resolved to janice by frequency |
| jenny | jennifer | |
| jerry | gerald | Also Jerome; resolved to gerald by frequency |
| jim | james | |
| jimmy | james | |
| joe | joseph | |
| johnny | john | |
| jon | jonathan | Distinct from john |
| kate | katherine | Also Kathryn, Catherine |
| kathy | katherine | |
| ken | kenneth | |
| kenny | kenneth | |
| larry | lawrence | |
| liz | elizabeth | |
| maggie | margaret | |
| matt | matthew | |
| mike | michael | |
| mitch | mitchell | |
| nancy | ann | Historical mapping; low reliability |
| nick | nicholas | |
| nikki | nicole | |
| norm | norman | |
| pat | patrick | Also Patricia; gendered ambiguity |
| patti | patricia | |
| patty | patricia | |
| peggy | margaret | |
| pete | peter | |
| phil | philip | |
| ray | raymond | |
| rick | richard | |
| rob | robert | |
| ron | ronald | |
| sally | sarah | |
| sam | samuel | Also Samantha; gendered ambiguity |
| sandy | sandra | Also Alexander; gendered ambiguity |
| steve | steven | |
| sue | susan | |
| ted | edward | See The Ted Problem below |
| terry | terrence | Also Teresa; gendered ambiguity |
| tim | timothy | |
| tom | thomas | |
| tommy | thomas | |
| tony | anthony | |
| val | valerie | |
| vince | vincent | |
| walt | walter | |
| wes | wesley | |
| will | william | |
| woody | woodrow | |

## The Ted Problem

"Ted" maps to both Edward (Ted Kennedy → Edward Kennedy) and Theodore (Ted Cruz → Rafael Edward Cruz, commonly Theodore). The dictionary maps `ted → edward` because Edward is the more frequent canonical form in US election data. This means a candidate whose legal name is Theodore but who files as Ted will be canonicalized as Edward.

This is a known, accepted error. It affects L1 `canonical_first` but does **not** prevent correct entity resolution at L3 — because L3 matches on composite strings that include last name, jurisdiction, office, and year. Two candidates named "Ted Smith" in different counties will not be merged regardless of whether `canonical_first` is edward or theodore.

The original filed name is always preserved in `first`. Any downstream consumer who needs the original can ignore `canonical_first` and use `first` directly.

## Gendered ambiguity

Several nicknames map to names that could be either male or female: Chris (Christopher/Christine), Pat (Patrick/Patricia), Sam (Samuel/Samantha), Sandy (Sandra/Alexander), Terry (Terrence/Teresa). The dictionary resolves these to the statistically more common canonical form in US election candidate data. The mapping is not always correct for individual candidates.

As with the Ted problem, the original name is preserved, and entity resolution at L3 uses additional fields (jurisdiction, office, party) to avoid incorrect merges caused by nickname ambiguity.

## When the dictionary is not applied

The dictionary is skipped when:

- The input first name is longer than 6 characters and matches no entry (assumed to already be a full name).
- The candidate record has a `canonical_first` value set by the source (some sources provide both nickname and legal name).
- The input is an initial only (e.g., "J." is not expanded).