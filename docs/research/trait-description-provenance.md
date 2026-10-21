# Trait description provenance

Reviewed: 2026-10-20. Source: PF2e 8.6.1 at
[`28d9d156f252d084371d994dc359509b49eadf18`](https://github.com/foundryvtt/pf2e/tree/28d9d156f252d084371d994dc359509b49eadf18).
Scope: the catalog and localized descriptions delivered in
[PR93](https://github.com/e-kosten/pf2e-atlas/pull/93).

## Disposition

The source path and applicable upstream notices are documented below. The pin
does **not** establish a paragraph-level license for each localized description.
This review therefore does not clear every description for use or redistribution.
The required content attribution/permission decision remains explicit in the
[separate follow-up](../backlog/items/trait-description-content-rights.md).

Atlas's Apache 2.0 license covers its own code. The generated catalog retains the
upstream system-code attribution; neither licenses the acquired rules prose as
Apache content. No content filtering, ingest change or new content-license grant
is introduced by this documentation.

## Source chain

1. The pinned [system catalog](https://github.com/foundryvtt/pf2e/blob/28d9d156f252d084371d994dc359509b49eadf18/src/scripts/config/traits.ts)
   supplies exact identifiers and explicit description-localization mappings.
2. [English localization](https://github.com/foundryvtt/pf2e/blob/28d9d156f252d084371d994dc359509b49eadf18/static/lang/en.json)
   supplies string values. Other indexing locales use the existing English
   fallback. Those strings do not carry individual book, author or license fields.
3. The source-contract extractor retains identifiers, keys, text, catalog
   memberships and source coordinates. The generated Rust catalog includes keys
   and source evidence, **not** localized prose. Ingest resolves and prepares
   selected descriptions into the local workspace.

The checked pin contains 784 trait identifiers: 664 have nonempty descriptions,
120 do not. There are 563 distinct mapped description keys; parameterized traits
can share a key. This is catalog coverage, not 664 independently licensed works.

| Example | Source evidence | What it establishes |
| --- | --- | --- |
| `agile` | `PF2E.TraitDescriptionAgile` | A mapped mechanics paragraph; the key alone does not identify a book or license. |
| `deadly-d8`, `deadly-d12` | `PF2E.TraitDescriptionDeadly` | Shared wording with separate trait identities; not independent attribution evidence. |
| `throneglass` | `PF2E.PreciousMaterialThroneglassDescription` | The paragraph combines setting history/proper names with mechanics. A trait description is not necessarily rules-only text. |
| `amp` | No description mapping | An honest missing description; no inferred source or license. |

## Notices and permission boundaries

The pinned [README](https://github.com/foundryvtt/pf2e/blob/28d9d156f252d084371d994dc359509b49eadf18/README.md)
distinguishes Apache-licensed implementation from content. It describes both
OGL mechanics and use under Foundry/Paizo's partnership. That partnership statement
does not establish permission granted to Atlas. The repository separately carries
an [OGL notice](https://github.com/foundryvtt/pf2e/blob/28d9d156f252d084371d994dc359509b49eadf18/static/licenses/OpenGameLicense.md)
and an [ORC attribution notice](https://github.com/foundryvtt/pf2e/blob/28d9d156f252d084371d994dc359509b49eadf18/static/licenses/ORCLicense.md).
Neither provides a mapping from localization keys to its listed publications.

The OGL requires identification of Open Game Content, applicable copyright
notices and a license copy; its Product Identity restriction is separate. The
pinned ORC notice lists source publications and reserves setting/proper-name
material, with no additional express designation of reserved material as licensed.
Their coexistence is not evidence that every paragraph is dual-licensed or that
legacy content can be reassigned to ORC. [Paizo's license guidance](https://paizo.com/licenses)
also distinguishes the two content grants from trademark permissions.

The existing Atlas Community Use notice is not paragraph-level attribution or a
blanket grant to copy books. The [Community Use Policy PDF](https://downloads.paizo.com/CommunityUsePolicy.pdf)
retrieved for this review is dated August 22, 2024; its permissions identify
particular material and descriptive uses, and require applicable credits.
Paizo also publishes a [Fan Content Policy](https://paizo.com/licenses/fancontent)
whose scope must be checked when selecting a permission basis. This review does
not silently replace Atlas's existing policy notice or presume those policies
cover every localization string.

## Distribution and remaining work

Current release tooling excludes reference databases and source/model caches.
Generated metadata and binaries do not compile in these description strings.
That reduces the material shipped in release archives; it does not resolve the
rights for preparation, local display, export or any future hosted/distributed
corpus. A local data cache is not itself licensing evidence.

Before making content-license claims, identify the publication and exact grant
for selected paragraphs, preserve the required notices/credits, and decide how
unknown or mixed rules/setting text is treated. Do not guess this from a trait
identifier, shared key, book date or mechanics/lore classifier. Any required
storage, ingest or content restriction belongs in the linked follow-up PR.

## Reproduction and evidence

Inspect the exact pin with the existing source-contract `extract --strict` command
and retain `trait-catalog.json` plus `summary.json` in ignored research output.
Count distinct identifiers only in catalogs whose namespace is `trait`; count
descriptions by nonempty resolved text and mappings by distinct non-null keys.
The upgrade comparison command documents which catalog bytes were compared;
its hashes are change/provenance evidence, not license classifications.

SHA-256 of the inspected pinned files:

| File | SHA-256 |
| --- | --- |
| `README.md` | `a76f3caa6f1cc36ba2145ee94ac5d53a0e09417e55c26eae118090613978250d` |
| `LICENSE` | `05013793d3f1c275201a62e544b117d0a946e76f02e7c840c371e51373a024e3` |
| `static/licenses/OpenGameLicense.md` | `a134a6a467427a1efa37026f958d53f432af78d1d4b4c4aff02f29f69a2bda31` |
| `static/licenses/ORCLicense.md` | `1e67784676bb566472d857b65e66126bc8b7259039f1d5f944f1fb7f8ee882ff` |
| `static/lang/en.json` | `f9442854d045a0e08fcdb5d80752c2010ded6614720b905e05e61124a39b9e8f` |
| `src/scripts/config/traits.ts` | `30d6cf6818ed091354afe3e61118233d8e1a2c58914ac613739d073ed90f7ad4` |

The pinned bytes were inspected locally; the primary Paizo/Foundry guidance was
checked online. Pinned GitHub file URLs were not retrievable through the browsing
cache. This is a source/attribution review, not a legal opinion or release clearance.
