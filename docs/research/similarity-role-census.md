# Similarity role and audience census

This is a **late current-acceptance census performed on October 27, 2026**. It does not establish that the complete typed mapping/audience census or ceiling freeze occurred before the functional preview. The earlier retained evidence includes a unit-size census and nine pilot measurements; no complete pre-preview role/audience receipt was located. The current checks close the substantive policy-coverage gap without backdating that evidence or changing the mapping recipe.

## Artifact and method

The read-only census observes generation `a9474928b73e8cf89e84c5ed6990a838:7`, schema `17`, source `28d9d156f252d084371d994dc359509b49eadf18`, content `foundry-content/v10`, selection `structural-source-sections/v4`, graph policy `typed-authored-occurrences/v6`, and role mapping `typed-content-correspondence/v1`. Both stored profiles, `bge-base` and `embeddinggemma`, are included. The product audience includes GM and owner content.

Stored metadata was read with SQLite URI `mode=ro`, `query_only=on`, and one explicit read snapshot. The developer diagnostic counted `similarity_parts`, joined stored units with the exact index-owned `read_similarity.rs::PART_JOIN`, and grouped prepared-field outcomes by typed family, owner depth, visibility and availability. It did not parse game prose, classify roles from field names, read vectors, mutate either workspace, or regenerate embeddings. Stored labels alone cannot verify the policy that produced them; the independent typed checks below supply that distinction.

The retained [compressed metadata](similarity-role-census/2026-10-27.json.gz) contains 63 role/field/owner buckets with bounded exact-address examples, 78 prepared-field audience/outcome buckets, per-profile unit availability and trait counts. It includes the source pin, reference generation, schema, source policy/test hashes, canonical join expression and acquisition timestamp. It contains no game-source bodies or vector values.

- Compressed SHA256: `a85e77e8a72fac736f1ebc3187f60802e670d7ec8fe000b3ba91610c09ba7237`
- Decoded canonical JSON SHA256: `97597a475ee9ebb287b73abca4582c3772bde77acf4506e09de5f9987c59ec9a`
- Base Git HEAD at read time: `a002dcfe242fe93b178094617e3597ef1069e624` (the acceptance test source was then uncommitted and is bound separately by SHA256).

## Observed counts

The artifact contains 29,894 checked source roots and 123,583 logical record sections. Of these sections, 38,846 belong to a root, 84,518 to an embedded owner and 219 to an owner nested two levels deep. Both profiles have stored units for all 123,583 sections; this is an inventory count, not a fresh native-vector validation.

| Stored comparison role | Logical sections |
| --- | ---: |
| ability | 47,999 |
| description | 60,734 |
| disable | 1,217 |
| journal_text | 5,482 |
| notes | 46 |
| prerequisites | 5,372 |
| reset | 694 |
| routine | 620 |
| stealth | 1,031 |
| table_result | 388 |

No record section in this artifact maps to `other`; a typed caption fixture verifies that policy branch independently. Trait definitions use their separate inventory: 784 definitions, with 668 containing prepared definition text and 116 label-only entries. Both profiles retain 784 trait vectors. Label-only trait behavior is additionally exercised by the app-service record/trait recipe-response test fixture.

Prepared rich fields total 148,126: 114,802 prepared and 33,324 empty. Their declared visibility is 140,493 All, 6,673 GM and 960 None. The None entries retain missing visibility rather than becoming public. These counts describe prepared-field metadata, not the number of semantic sections or the number visible to a public-only caller. Plain fields and absent authored fields do not have a prepared-rich-field row; their selection is covered by typed checks.

## Independent policy checks

[The typed policy matrix](../../crates/atlas-record/src/source_record/similarity_tests.rs) starts with admitted source DTOs and uses the real `prepare_record_content`, `select_record_search`, and `similarity_content_role` owners. It verifies:

- Action, feat and effect descriptions become Ability for both root and embedded copies; identical spell, equipment, weapon, armor, consumable, ancestry, class, deity and kit prose remains Description. The text itself does not choose the role.
- Public NPC notes become Description, private NPC notes remain Notes, and embedded private ability prose stays audience-gated. Empty public/embedded bodies and whitespace-only blurbs do not produce sections.
- Character appearance/backstory use Description, while campaign/personality biography uses Notes. Independent public/owner flags and missing visibility are respected.
- Journal page text, image captions and table results retain Journal text, Other and Table result roles; empty journal pages contribute no text section.
- Hazard description, routine, disable, reset and stealth retain distinct roles, and feat prerequisites remain Prerequisites.

Five matrix tests pass. Additional focused app-service checks serialize both record and label-only trait responses for baseline and passage methods, acknowledging the canonical scoring and mapping recipes. Existing index/search coverage checks role eligibility before KNN, profile isolation, missing roles and trait label-only stored operation. The [architecture decision](../architecture/decisions/0059-configurable-similarity.md) remains the policy owner; this file records evidence rather than defining another mapping.

The compact metadata binds policy matrix SHA256 `905b37da1eec0250cfca2bfb2cf5521ebf974b901fa84518f2455eb8831f7bcc`. The focused receipts and diagnostic source are retained in the task's `scratch/similarity-evaluation/final-gates/backend-remediation/`; broader final gates and browser/CLI acceptance are separate candidate evidence.
