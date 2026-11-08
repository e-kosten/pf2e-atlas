import type * as Models from "./http";

export type * from "./http";

export type StrikeRangeView = NonNullable<Models.FactViewStrikeRangeView["value"]>;
export type ActionCostView = NonNullable<Models.FactViewActionCostView["value"]>;
export type ActorModifierView = NonNullable<
  Models.FactViewVecActorModifierView["value"]
>[number];
export type ActorMovementView = NonNullable<
  Models.FactViewVecActorMovementView["value"]
>[number];
export type ActorSenseView = NonNullable<
  Models.FactViewVecActorSenseView["value"]
>[number];
export type ActorSkillView = NonNullable<
  Models.FactViewVecActorSkillView["value"]
>[number];
export type ConditionalSkillView = NonNullable<
  Models.FactViewVecConditionalSkillView["value"]
>[number];
export type DamageComponentView = NonNullable<
  Models.FactViewVecDamageComponentView["value"]
>[number];
export type FactView<T> = Omit<Models.FactViewString, "value"> & { value: T | null };
export type GraphContextView = Extract<
  Models.RelatedCategoryViewGraphContextView,
  { kind: "content" }
>["content"];
export type HeighteningDamageView = NonNullable<
  Models.FactViewVecHeighteningDamageView["value"]
>[number];
export type IwrEntryView = NonNullable<Models.FactViewVecIwrEntryView["value"]>[number];
export type PhysicalPriceView = NonNullable<Models.FactViewPhysicalPriceView["value"]>;
export type SpellAreaView = NonNullable<Models.FactViewSpellAreaView["value"]>;
export type SpellChangeView = NonNullable<Models.FactViewSpellChangeView["value"]>;
export type SpellDefenseView = NonNullable<Models.FactViewSpellDefenseView["value"]>;
export type SpellFixedLevelView = NonNullable<
  Models.FactViewVecSpellFixedLevelView["value"]
>[number];
export type SpellFormView = NonNullable<
  Models.FactViewVecSpellFormView["value"]
>[number];
export type SpellHeighteningChangeView = NonNullable<
  Models.FactViewSpellHeighteningChangeView["value"]
>;
export type SpellHeighteningView = NonNullable<
  Models.FactViewSpellHeighteningView["value"]
>;
export type RelatedCategoryView<T> =
  | Exclude<Models.RelatedCategoryViewGraphContextView, { kind: "content" }>
  | (Omit<
      Extract<Models.RelatedCategoryViewGraphContextView, { kind: "content" }>,
      "content"
    > & { content: T });
export type ConsumableDamageView = NonNullable<
  Models.FactViewConsumableDamageView["value"]
>;
export type WeaponDamageReferenceView = NonNullable<
  Models.FactViewWeaponDamageReferenceView["value"]
>;
export type WeaponPersistentReferenceView = NonNullable<
  Models.FactViewWeaponPersistentReferenceView["value"]
>;
export type WeaponAmmunitionReferenceView = NonNullable<
  Models.FactViewWeaponAmmunitionReferenceView["value"]
>;
export type WeaponRuneReferenceView = NonNullable<
  Models.FactViewWeaponRuneReferenceView["value"]
>;
export type KitEntryReferenceView = NonNullable<
  Models.FactViewVecKitEntryReferenceView["value"]
>[number];
export type AttributeChoiceView = NonNullable<
  Models.FactViewVecAttributeChoiceView["value"]
>[number];
export type ReferenceGrantView = NonNullable<
  Models.FactViewVecReferenceGrantView["value"]
>[number];
export type RemasterRelationshipsView = Extract<
  Models.RelatedCategoryViewRemasterRelationshipsView,
  { kind: "content" }
>["content"];
export type RemasterSourceContextView = Extract<
  Models.RelatedCategoryViewRemasterSourceContextView,
  { kind: "content" }
>["content"];

// UI builders provide these normalized fields. The generated HTTP calls
// also accept the omitted fields that Serde defaults for external callers.
type AppFields<T, Fields> = T extends unknown
  ? { [K in keyof T]: K extends keyof Fields ? Fields[K] : T[K] }
  : never;

export type RecordDetailRequest = AppFields<
  Models.RecordDetailRequest,
  { passage: Models.SourcePassageAddress | null }
>;
export type EmbeddedReadingRequest = AppFields<
  Models.EmbeddedReadingRequest,
  { parent: Models.RecordNavigationView }
>;
export type SearchCriteria = AppFields<
  Models.SearchCriteria,
  { filter: Models.QueryPredicate | null; similarity: Models.SimilarityOptions | null }
>;
export type SearchResultsRequest = AppFields<
  Models.SearchResultsRequest,
  { criteria: SearchCriteria }
>;
export type FilterDiscoveryContext = AppFields<
  Models.FilterDiscoveryContext,
  { filter: Models.QueryPredicate | null }
>;
