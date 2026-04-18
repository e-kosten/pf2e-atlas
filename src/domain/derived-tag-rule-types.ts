import type { SearchCategory, SearchSubcategory } from "./search-types.js";

export type DerivedTagAuthoredRuleIntent = "deterministic";
export type DerivedTagAuthoredRuleKind =
  | "trait_match"
  | "family_match"
  | "reference_match"
  | "composite_tag"
  | "exact_text_match"
  | "text_context_match";

export type DerivedTagAuthoredTextScope = "either" | "name" | "description" | "blurb";

export type DerivedTagAuthoredTextValue =
  | string
  | {
      value: string;
      scope?: DerivedTagAuthoredTextScope;
    };

export interface DerivedTagAuthoredReferencePredicate {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  packName?: string;
  nameAny?: string[];
  traitsAny?: string[];
  traitsAll?: string[];
}

export interface DerivedTagAuthoredTextNearConstraint {
  all: DerivedTagAuthoredTextValue[];
  window: number;
  ordered?: boolean;
  scope?: DerivedTagAuthoredTextScope;
}

export interface DerivedTagAuthoredRuleBase {
  tag: string;
  category: SearchCategory;
  subcategories?: SearchSubcategory[];
  intent: DerivedTagAuthoredRuleIntent;
  description?: string;
}

export interface DerivedTagAuthoredRuleBlockers {
  traitsAny?: string[];
  traitsAll?: string[];
  familiesAny?: string[];
  familiesAll?: string[];
  textAny?: DerivedTagAuthoredTextValue[];
  textAll?: DerivedTagAuthoredTextValue[];
  textNear?: DerivedTagAuthoredTextNearConstraint[];
  referencesAny?: string[];
  referencesAll?: string[];
  referencesWhere?: DerivedTagAuthoredReferencePredicate[];
  minReferenceMatches?: number;
}

export interface DerivedTagAuthoredTraitMatchRule extends DerivedTagAuthoredRuleBase {
  kind: "trait_match";
  when: {
    traitsAny?: string[];
    traitsAll?: string[];
  };
  blockers?: DerivedTagAuthoredRuleBlockers;
}

export interface DerivedTagAuthoredFamilyMatchRule extends DerivedTagAuthoredRuleBase {
  kind: "family_match";
  when: {
    familiesAny?: string[];
    familiesAll?: string[];
  };
  blockers?: DerivedTagAuthoredRuleBlockers;
}

export interface DerivedTagAuthoredReferenceMatchRule extends DerivedTagAuthoredRuleBase {
  kind: "reference_match";
  when: {
    referencesAny?: string[];
    referencesAll?: string[];
    referencesWhere?: DerivedTagAuthoredReferencePredicate[];
    minReferenceMatches?: number;
  };
  blockers?: DerivedTagAuthoredRuleBlockers;
}

export interface DerivedTagAuthoredCompositeTagRule extends DerivedTagAuthoredRuleBase {
  kind: "composite_tag";
  when: {
    anyTags: string[];
  };
}

export interface DerivedTagAuthoredExactTextMatchRule extends DerivedTagAuthoredRuleBase {
  kind: "exact_text_match";
  when: {
    textAny?: DerivedTagAuthoredTextValue[];
    textAll?: DerivedTagAuthoredTextValue[];
  };
  blockers?: DerivedTagAuthoredRuleBlockers;
}

export interface DerivedTagAuthoredTextContextMatchRule extends DerivedTagAuthoredRuleBase {
  kind: "text_context_match";
  when: {
    textNear: DerivedTagAuthoredTextNearConstraint[];
  };
  blockers?: DerivedTagAuthoredRuleBlockers;
}

export type AuthoredDerivedTagRule =
  | DerivedTagAuthoredTraitMatchRule
  | DerivedTagAuthoredFamilyMatchRule
  | DerivedTagAuthoredReferenceMatchRule
  | DerivedTagAuthoredCompositeTagRule
  | DerivedTagAuthoredExactTextMatchRule
  | DerivedTagAuthoredTextContextMatchRule;
