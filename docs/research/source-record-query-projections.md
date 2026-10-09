# Initial query projection catalog and current surface inventory

The callable library in `atlas-record::source_record` exposes these initial
typed views over the retained Foundry DTO. IDs below describe semantic intent;
they are not published CLI syntax or a runtime discovery registry. Runtime
catalog/SQL/UI/CEL integration belongs to the later artifact cutover. See
[ADR 0045](../architecture/decisions/0045-source-backed-record-enrichment.md).

The authoritative record contains only its key and DTO. Existing focused query
views dispatch on generated source variants; they do not require stored
embedded-node/container inventories or a second family-view hierarchy. Broader
domain accessors and public traversal/lookup APIs are deferred to concrete
consumer work.

## Initial field set

| Field intent | Type / units | Initial applicability and meaning |
| --- | --- | --- |
| record.kind | Existing Atlas RecordKind | Explicit family-to-product classification; retain unresolved classification instead of guessing from names |
| source.document_kind | Root kind | Actor, Item, JournalEntry, Macro, RollTable from matched source variant |
| source.type | Source discriminator | Authored Actor/Item family or Macro type where declared; distinct from root kind |
| source.pack.id | String | Loader pack identity, all retained addressed roots |
| source.pack.label | String | Loader pack label, all retained addressed roots |
| publication.title | String | Typed publication fields where generated family exposes them |
| publication.remaster | Boolean | Authored remaster field; absent is unavailable, not false |
| traits | Open string set | Authored traits.value where family exposes it; otherTags stays a separate source concept |
| rarity | Generated rarity vocabulary | Applicable family traits.rarity; no implicit common default |
| actor.level | Number / authored level | NPC and hazard details.level.value initially |
| actor.armor_class | Number / authored AC | NPC and hazard attributes.ac.value initially |
| actor.hp.maximum | Number / HP | NPC and hazard attributes.hp.max; independent of current snapshot HP |
| hazard.hardness | Number / hardness | Authored hazard attributes.hardness |
| hazard.complexity | Boolean | Authored hazard details.isComplex; false remains known false |
| spell.rank | Number / base rank | Authored Spell system.level.value; no heightening/prepared-slot substitution |
| spell.traditions | Declared identifier set | Authored Spell traits.traditions |
| actor.items | Owned Item collection | Family-dispatched authored Actor items with explicit ancestor/collection availability and owned identity; no runtime-prepared child availability |

Within actor.items, reuse child source.type, traits, spell.rank and
spell.traditions semantics with that child's applicability. Root and child scopes
remain explicit. Fixtures prove one owned spell matching family=spell, rank>=3
and trait=fire; separate child witnesses cannot satisfy the conjunction. Keep
all owned Item families/addressable bodies, not only spell children. Invalid
collections and unknown child fields cannot prove known absence.

Native source Number is retained; no arbitrary conversion of all numeric data to
anonymous f64 rows. Every intermediate SourcePresence contributes to
availability. Closed traits vocabularies are not introduced. This checklist does
not register every source field or demand a source-field coverage receipt.

Source evidence and owners:

- generated NPC: `crates/atlas-foundry-model/src/source_model/generated/actors/families/npc.rs` (attributes/details/traits/items).
- generated hazard: same subtree `hazard.rs` (attributes/details/traits/items).
- generated Spell: `crates/atlas-foundry-model/src/source_model/generated/items/families/spell.rs` (level/traits/publication).
- shared publication/trait owners: generated actors/common.rs and items/common.rs.
- admission states: `crates/atlas-foundry-model/src/source_model/presence.rs`.
- pack labels/provenance: atlas-ingest/source_loading loader/model; do not infer from document names.

The enriched ingest result retains pack labels alongside document outcomes;
SourceFileProvenance alone supplies pack name, not the label. These two metadata
filters use explicit source-pack context, independent from the body DTO.

Text-source views borrow names, publication/trait values and other selected plain
labels directly, then combine them with explicitly prepared rich text. They retain
root/owner/field attribution and audience visibility. Content preparation and
structured relationship resolution return separate outputs composed by ingest;
recognized prose references remain with prepared content. Missing/null/invalid
text remains source availability/admission evidence, while actual rich text with
unsupported format or failed preparation has an explicit outcome. No exhaustive
optional-field inventory is part of query semantics.

At artifact adoption, FTS text and document embedding inputs use an English-default,
user-overridable indexing locale recorded with relevant localization identity.
Changing search locale requires re-indexing; initial display uses that artifact
context. A future display locale does not implicitly change search. The library
keeps localization input explicit but publishes no locale CLI flag, storage key
or UI setting. Detailed ingest/preparation reporting stays in developer tools,
not normal query result DTOs.

## Existing metadata/filter inventory

Complete current metadata enums are in atlas-domain/src/metadata.rs: 14 set,
15 enum/string, 7 text, 7 number and 4 Boolean categories. Record-kind and graph
filters are additional request concepts. Current discovery definitions are in
atlas-index/src/discovery/definitions.rs. Proposed meanings below account for all
categories; the current production surface stays unchanged in this library PR.

**Initial** means a callable projection is included now. **Deferred** means
retained source and later expansion; it does not authorize dropping a current
product filter. **Decision** means semantics need explicit review before cutover.

| Current category | Disposition |
| --- | --- |
| RecordKind, PackName, PackLabel, FoundryRecordType | Initial; separate document kind and authored type |
| Traits, Traditions, Rarity, PublicationTitle, PublicationRemaster, IsComplex | Initial for declared/applicable families |
| Level | Initial NPC/hazard level and spell base rank; other family levels deferred; public generic-level migration needs explicit review |
| SpellKinds | Deferred typed spell/ritual vocabulary; preserve distinctions, no inference from name |
| DamageTypes | Deferred contextual damage projections; preserve source damage components, scope and member availability |
| Languages, SpeedTypes, Senses, Immunities, Resistances, Weaknesses | Deferred named typed collections with conditions/units and collection availability |
| Size, Usage, SystemCategory, SystemGroup, BaseItem, Hands(enum) | Deferred family-specific source accessors/vocabulary |
| SaveType, AreaType, BasicSave, Sustained | Deferred source fields with explicit null/availability semantics |
| RangeText, DurationText, TargetText, DisableText | Deferred dedicated filters; selected prose still participates in source-backed content/text |
| PriceCp | Decision: amount/denomination defaults/positive quantity basis/per-item versus bundle comparison |
| BulkValue | Decision: authored bulk, light/negligible vocabulary and prepared size/container effects |
| ActionCost | Decision: named economy, free/reaction/passive and authored spell time |
| RangeValue, AreaValue, DurationUnit | Decision: source string/numeric unions and unit-aware comparisons; no text-prefix parsing |
| PublicationCategory, TaxonomyFamilies | Decision: explicit Atlas grouping/classification policy |
| VariantGroupKey, VariantAxes, VariantBaseName, VariantLabel | Decision: grouping identity/verified relation policy; do not inherit heuristics automatically |
| DisableSkills | Decision: interpreted alternatives/associations in prose; recognized Check alone is not an authoritative disable skill/rank model |
| DerivedTags | Currently rejected by SQL compiler; separate tag assignment/catalog work |
| Hands(number) | Currently rejected by SQL compiler; enum Hands is the functioning filter |
| LinksTo, LinkedFrom | Initial complete outgoing occurrence facts; inverse query/graph-policy SQL belongs to artifact/index adoption |
| Boolean grouping/negation, MetricCompare | Backend semantics/operators, not stored source fields; shared compiler later |

Unsupported current entries are explicitly rejected in
atlas-index/src/read/search/filters/metadata.rs (DerivedTags and numeric Hands).
They should not be described as working features removed by the new design.

## Existing 35 metric definitions

Complete list: atlas-record/src/metrics/matching.rs. Actor22/Item13 definitions,
including parameterized key families. Audit categories below cover the complete
list. Keep old production extraction until the coherent replacement; do not
bridge new source projections back into old keys or create compatibility aliases.

| Current metric keys | Disposition |
| --- | --- |
| ac.value, hp.max | Initial NPC/hazard authored baselines |
| hardness.value | Initial hazard; other family scope deferred |
| hp.value | Deferred authored snapshot current; separate from maximum and encounter participant state |
| hp.bt | Decision: explicit verified threshold derivation |
| perception.mod | Deferred NPC authored baseline |
| stealth.mod | Deferred family-specific authored stealth |
| stealth.dc | Decision: authored versus converted/prepared detection DC |
| ability.{ability}.mod, save.{save}.mod | Deferred typed family-specific authored basis |
| save.best, save.worst | Decision: complete inputs and tied result sets |
| skill.{skill}.mod | Deferred NPC authored base; conditional variants and Lore retain context |
| skill.{skill}.rank, skill.{skill}.proficient | Decision: authored family-specific proficiency; no inferred NPC convention |
| speed.{movement}.value, sense.{sense}.range | Deferred units and optional/unavailable sense range |
| disable.dc.min, disable.dc.max, disable.{skill}.dc.min, disable.{skill}.dc.max, disable.{skill}.rank.min | Decision: prose alternatives/skill-rank association/aggregation policy |
| armor.ac_bonus, armor.dex_cap, armor.strength, armor.check_penalty, armor.speed_penalty | Deferred typed armor fields with units |
| shield.ac_bonus, shield.hardness, shield.hp | Deferred authored shield values/current-capacity distinction |
| shield.bt | Decision: verified threshold policy |
| weapon.range_increment | Deferred unit-aware range |
| weapon.reload | Decision: vocabulary/empty/dash values and numeric interpretation |
| weapon.damage_dice, weapon.damage_die_faces | Deferred typed damage components/die vocabulary |

The concrete same-attack fire/bonus/melee case remains future
catalog expansion. A Foundry Melee Item can be ranged; family=melee is not attack
mode. Its pinned classification checks range-/thrown- traits, which require a
complete valid trait collection. This initial plan uses the simpler owned-spell
scope proof and does not substitute document family for attack mode.

## Expansion and product cutover rules

Add a new supported-shape field through typed access/derivation, one catalog
binding, an index assessment and focused tests. Add a new relationship/operator
once in the shared backend. Existing DTOs retain unprojected fields, so expansion
does not require another family-schema or ingest rewrite.

Before product cutover, review this inventory with the user: every functioning
existing filter is retained, renamed with clear semantics, or explicitly retired.
Deferred in this library plan is not a decision to ship a regression. CLI CEL
and structured UI input share backend semantics but do not translate into one
another. Prove SQL availability/negation, same-child filtering/facets and scope
before vector top-k when the database/compiler work is implemented.
