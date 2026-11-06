# Remaster information: coverage and model review

Research date: 2026-11-05. Atlas baseline: `344395cf917c840f7b2c10469c19200e2bbf4c16`
(merged PR #213). Foundry source: `28d9d156f252d084371d994dc359509b49eadf18`,
PF2e 8.6.1, English. This report separates observed behavior from proposed changes.
This baseline report predates the implementation now
[in progress](../backlog/items/remaster-relationship-coverage.md). See the
[2026-11-06 counterpart audit](remaster-counterpart-coverage.md) for current
declaration/alias counts, available but unbound old records and comparison coverage.

## Finding

Atlas has useful publication metadata and conservative identity boundaries, but
its alias/pair projections do not preserve the remaster information available in
the source. The principal gap is a missing model for an **authored change
statement**. A statement can describe a renamed term, merged options, changed
mechanics, a removed option, a group of replacements, or an explicit ID redirect.
It can remain useful when an older document is absent.

Extending `remaster_pairs` alone would leave those cases unrepresented. A bounded,
typed declaration model with separate lookup and search-preference projections is
the recommended direction. A general-purpose graph framework is not justified by
this investigation.

## What is currently present

Read-only queries against the retained full-corpus development workspace found:

| Observed fact                         |  Count |
| ------------------------------------- | -----: |
| Root records                          | 29,894 |
| Explicit publication `remaster=true`  | 19,488 |
| Explicit publication `remaster=false` |  9,956 |
| Publication flag not applicable       |    450 |
| Verified name aliases                 |    495 |
| Distinct normalized alias terms       |    482 |
| Alias terms with two target records   |     13 |
| Stored remaster pairs                 |  **0** |

The database's recorded source revision matches the pin above. Its source
fingerprint is `dcc3a33936e7e519dcded488fc102c53e6dab8fce74a9a3c0b803619b9c640f7`,
locale `en`, relationship policy `typed-authored-occurrences/v7`. These are
observed corpus counts, not permanent acceptance totals for every future source
revision. No installed workspace was inspected or modified.

[PR #213](https://github.com/e-kosten/pf2e-atlas/pull/213) added publication labels
and an above-rules counterpart notice. It deliberately did not change ingest.
Current root details expose verified pairs; embedded details do not inherit the
parent's pair. Pair-reading failures remain secondary to the record read.

Publication era is not a compatibility verdict. Foundry's
[publication field](https://github.com/foundryvtt/pf2e/blob/28d9d156f252d084371d994dc359509b49eadf18/src/module/model.ts#L42)
defaults to false; historical
[migration 877](https://github.com/foundryvtt/pf2e/blob/28d9d156f252d084371d994dc359509b49eadf18/src/module/migration/migrations/877-publication-data.ts)
initialized it using publication titles and a fallback. It is also editable.
An explicit false is a source fact; it does not prove obsolete rules, an available
replacement, or incompatibility with remaster play. Paizo describes the remaster
as the same core system, with broadly compatible existing material, in its
[FAQ](https://paizo.com/pathfinder/remaster/faq). Individual changes still require
specific evidence.

## Source coverage

The pinned [Remaster Changes journal](https://github.com/foundryvtt/pf2e/blob/28d9d156f252d084371d994dc359509b49eadf18/packs/pf2e/journals/remaster-changes.json)
contains 560 data rows: 31 class features, 210 feats, 146 spells, 90 equipment
entries and 83 bestiary entries. There are 566 HTML table rows including five
headers and an empty row. Independent bounded HTML-parser probes agreed on these
counts.

| Authored status or table form | Rows |
| ----------------------------- | ---: |
| Renamed                       |  272 |
| Merged                        |   50 |
| Removed                       |   54 |
| Altered mechanics             |    5 |
| Renamed and altered mechanics |    3 |
| Replaced                      |    2 |
| `Rename` spelling variant     |    1 |
| Two-column name-change rows   |  173 |

The current [extractor](https://github.com/e-kosten/pf2e-atlas/blob/344395cf917c840f7b2c10469c19200e2bbf4c16/crates/atlas-ingest/src/source_aliases.rs) reduces
that source through this funnel:

1. 504 rows pass the initial positional/status gate. This includes seven
   three-column Removed spell rows because only four-column rows receive the
   status check. All 504 old cells contain plain names rather than root references.
2. 488 have a single destination UUID. The other 16 have no destination or multiple
   destinations. The seven Removed spell rows have no target and disappear here.
3. Two destinations are embedded journal pages rather than root records: 486 remain.
4. Same-normalized-name filtering removes the Skillful Tail declaration: 485 table
   aliases remain.
5. Nine introductory ancestry/heritage aliases produce 494 journal aliases.
6. The recognized Flat-Footed migration comment adds one, matching the 495 stored
   aliases. No row supplies both explicit root endpoints, so no pair is emitted.

This explains the empty pair table without proving that the source has no remaster
relationships. It also shows why simply increasing pair counts is not a safe goal.

Important declarations currently lost or flattened include:

| Example                                           | Information that must survive                                                                                  |
| ------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Abundant Step                                     | A feat merges into Advanced Qi Spells; a spell is renamed Shrink the Span. Family/context distinguishes them.  |
| Stance Savant                                     | Fighter and Monk declarations have different successors.                                                       |
| Feather Token variants; Wyrm on the Wing variants | Each row groups three destinations. The grouping does not assert every old/new combination.                    |
| Skillful Tail (Ganzi)                             | A same-name Merged declaration remains meaningful despite not adding a useful alias.                           |
| Speedy Rituals                                    | Marked Removed, with an explicit reference to Efficient Rituals. Removal can coexist with a related successor. |
| Cauldron; Crossbow Ace                            | Changed mechanics can be explained without a distinct old document.                                            |
| Languages, domains, pantheons and materials       | Respectively 12, 3, 4 and 2 name changes have no record UUID destination.                                      |
| Witch patrons and wizard schools                  | Replacement context links to a journal discussion, not one interchangeable item.                               |

The maintained [Foundry wiki](https://github.com/foundryvtt/pf2e/wiki/Remaster-Changes)
also distinguishes these categories and describes its coverage as partial. It is
inspiration and corroboration; the pinned source supplies reproducible Atlas inputs.

### Additional inputs and missing documents

[Migration 887](https://github.com/foundryvtt/pf2e/blob/28d9d156f252d084371d994dc359509b49eadf18/src/module/migration/migrations/887-redirect-spell-links.ts)
contains six explicit old/new spell-ID mappings. All six old records are absent
from the inspected main corpus; every target is present. Dancing Lights points
to Light; Remove Curse and Remove Disease both point to Cleanse Affliction.
These are explicit redirect/replacement declarations, not proof of unchanged
mechanics. The current migration reader recognizes one comment pattern and misses
this structured mapping.

[Migration 850](https://github.com/foundryvtt/pf2e/blob/28d9d156f252d084371d994dc359509b49eadf18/src/module/migration/migrations/850-flat-footed-to-off-guard.ts)
provides an in-place Flat-Footed to Off-Guard rename with the retained condition ID.
A name-history fact therefore need not connect two distinct records.

The pin's `build/uuid-redirects/pf2e.json` contains 1,515 redirects, including pack
moves, consolidation, self-redirects and adventure variants. Only five old IDs
are present in the current corpus. None identifies the Attack of Opportunity
glossary relationship or migration 887's six old IDs. These redirects are
identity/transport history; they cannot all be classified as remaster changes.

Foundry distributes older snapshots separately in the
[Legacy Content module](https://foundryvtt.com/packages/pf2e-legacy-content).
That module was not in the inspected source pin or Atlas source cache. Adding it
would require a separately pinned/admitted provider and identity policy. This
research did not import it or inspect the user's Foundry installation. Historical
Magic Missile ID continuity was not established from the shallow source checkout.

### Attack of Opportunity is a boundary case

The retained legacy glossary key is
`bestiary-ability-glossary-srd:hFtNbo1LKYCoDy2O`; the modern glossary Reactive
Strike is `bestiary-ability-glossary-srd:W7SbTykXrNwxDzJc`. Their localized English
bodies are byte-identical at this pin. Their authored localization keys differ.
The shared localized-body SHA-256 is
`6366c8de03d73b324eae84473f9008fd53573ebd1dd13e871efbf57f238eedd9`.

However, the journal's Class Features row points to the **general action**
`actionspf2e:KAVf7AmRnbCAHrkT`, not the modern glossary. The corpus also contains
feat/class-feature Reactive Strike records. No explicit pinned ID assertion
connecting the two glossary records was found.

A same-name/family match would choose the wrong context here. Matching body text
is useful review evidence, not a general identity rule. A reviewed, source-bound
Atlas correspondence could connect the glossary records if that evidence class
is accepted; it must remain distinguishable from an upstream-authored mapping.

Gelugon's owned Attack of Opportunity cites the legacy glossary through
`_stats.compendiumSource`. Its owned identity and any customized content remain
separate from the general source entry and its remaster context.

## Adversarial storage and policy review

Independent read-only review confirmed these limitations. They are existing or
latent limitations; zero stored pairs means the pair hazards were not observed
as production corpus failures.

1. **Evidence multiplicity conflicts with publication.** Extraction emits one
   input per distinct citation. The [writer](https://github.com/e-kosten/pf2e-atlas/blob/344395cf917c840f7b2c10469c19200e2bbf4c16/crates/atlas-index/src/publication_delta.rs#L148)
   rejects a second input with the same legacy key, while the
   [schema](https://github.com/e-kosten/pf2e-atlas/blob/344395cf917c840f7b2c10469c19200e2bbf4c16/crates/atlas-index/migrations/00000000000001_create_artifact/up.sql#L544)
   has that key as its primary key. Two citations supporting the same pair can
   therefore fail publication. Alias extraction distinguishes evidence strings,
   while its writer rejects duplicate `(record, alias)` inputs as well.
2. **One legacy key has one destination and one citation.** Existing pairs allow
   many old records to share one target, but cannot retain fan-out, conflicting
   assertions or multiple citations. A pair also requires both documents to exist.
3. **Meaning and authority are conflated.** Renamed, Merged and Replaced flow into
   the same pair contract; search uses every pair as suppression authority. A
   relationship or alias does not establish interchangeable content.
4. **Cycles are admitted.** Only self-edges are forbidden. With `A→B` and `B→A`,
   independent matching-endpoint suppression can hide both results. Conflicts and
   cycles must be handled before broader evidence enables preference edges.
5. **A citation is not a checked assertion.** Current deep validation checks typed
   source locations and field availability, but not that a particular statement
   establishes the claimed endpoint binding. Alias provenance is an opaque string.
6. **Saved-list preference is inconsistent across views.** Unfiltered lists read
   exact keys and show both versions. Filtered views and lexical list search apply
   preference, temporarily hiding an explicitly saved legacy member when its
   saved counterpart also matches. This is covered by an existing
   [regression](https://github.com/e-kosten/pf2e-atlas/blob/344395cf917c840f7b2c10469c19200e2bbf4c16/crates/atlas-app-service/src/result_order_tests.rs#L165), not
   an accidental untested path. The new plan should explicitly change that policy.
7. **Human CLI and user controls lag the DTO.** Record DTOs carry edition and
   counterpart context, but human record output omits it. Ordinary search
   hardcodes remaster preference; its app request exposes no user choice.

Retain the strengths: explicit key and strict ambiguous-name resolution do not
substitute a remaster; suppression uses complete request membership before paging;
ordinary search does not decode unrelated provenance; exact source-field
navigation and secondary detail errors already exist; encounters retain saved
record identities.

## Recommendation and presentation direction

Preserve publication facts independently. Store typed declarations with ordered
sides, contextual qualifiers, source-bound evidence, compound change kinds and
explicit endpoint resolution. Derive name lookup and narrowly approved preference
edges from them. Preserve grouped assertions rather than expanding a Cartesian
record graph. Keep absence, ambiguity, contradiction and unavailable data distinct.

A smaller patch adding a relation kind and child citations to `remaster_pairs`
would fix multiplicity, but still lose absent endpoints, same-record history,
terms, removals and grouped statements. It is insufficient for the requested
scope. A general graph adds traversal machinery without resolving evidence or
consumer authority. The typed declaration approach is the preferred middle ground.

For presentation, retain the compact publication badge and actionable context above
rules. Use relation-specific language: former name, merged into, replacement
options, changed mechanics, or verified version. For an authored rename, missing
old text can still support an attributed former-name fact rather than a fabricated
record link. Absence alone establishes no rename; mergers and replacements keep
their own meaning. A separate Legacy label should never imply incompatibility.

AoN provides useful examples: [Magic Missile](https://2e.aonprd.com/Spells.aspx?ID=180&NoRedirect=1)
and [Force Barrage](https://2e.aonprd.com/Spells.aspx?ID=1536&NoRedirect=1) expose
reciprocal navigation before the rules. Its distinct [legacy Divine Lance](https://2e.aonprd.com/Spells.aspx?ID=84&NoRedirect=1)
and [remaster Divine Lance](https://2e.aonprd.com/Spells.aspx?ID=1498&NoRedirect=1)
pages demonstrate why equal names do not establish equal content. Atlas should
preserve exact reading URLs and use its existing drawer/history rather than
redirecting the chosen document.

The local working plan owns proposed schema details, execution order, acceptance
cases and unresolved product choices. The [backlog item](../backlog/items/remaster-relationship-coverage.md)
owns durable scope. Additional reference providers, a general identity-redirect
system, and explicit saved-entry conversion are separate potential workstreams.

Product decision on 2026-11-05: saved records follow current content at their
selected key. Historical record snapshots and pinning are excluded. The proposed
model therefore keeps current records, supported change statements and derived
current bindings/policy; it does not require document-version or assessment
history. Current source evidence remains necessary for attribution and validation.
