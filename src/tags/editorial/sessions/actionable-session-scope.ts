import type { SearchCategory } from "../../../domain/index.js";
import { DERIVED_TAG_MANAGED_CATEGORIES } from "../../manifest.js";
import { normalizeDerivedTag } from "../../runtime/matcher/shared.js";
import { getCurrentDerivedTagMigrationAuthoredState } from "../state/authored-state.js";
import { getPublishedDerivedTagMigrationOntology } from "../state/runtime-state.js";
import type { DerivedTagMigrationMode } from "../types.js";

export type DerivedTagActionableSessionScopeKeys = {
  familyKeys: Set<`${SearchCategory}:${string}`>;
  tagKeys: Set<`${SearchCategory}:${string}`>;
};

function buildScopeKey(category: SearchCategory, value: string): `${SearchCategory}:${string}` {
  return `${category}:${normalizeDerivedTag(value)}`;
}

function createEmptyScopeKeys(): DerivedTagActionableSessionScopeKeys {
  return {
    familyKeys: new Set<`${SearchCategory}:${string}`>(),
    tagKeys: new Set<`${SearchCategory}:${string}`>(),
  };
}

function addTag(keys: DerivedTagActionableSessionScopeKeys, category: SearchCategory, tag: string): void {
  const normalizedTag = normalizeDerivedTag(tag);
  keys.tagKeys.add(`${category}:${normalizedTag}`);

  const ontology = getPublishedDerivedTagMigrationOntology();
  const ontologyTag = ontology.tagByKey.get(`${category}:${normalizedTag}`);
  if (!ontologyTag) {
    return;
  }

  keys.familyKeys.add(buildScopeKey(category, ontologyTag.family));
}

function addFamily(keys: DerivedTagActionableSessionScopeKeys, category: SearchCategory, family: string): void {
  keys.familyKeys.add(buildScopeKey(category, family));
}

export function matchesDerivedTagFamilyFilter(
  category: SearchCategory,
  tag: string,
  family: string | undefined,
): boolean {
  if (!family) {
    return true;
  }

  const ontology = getPublishedDerivedTagMigrationOntology();
  const ontologyTag = ontology.tagByKey.get(buildScopeKey(category, tag));
  if (!ontologyTag) {
    return false;
  }

  return normalizeDerivedTag(ontologyTag.family) === normalizeDerivedTag(family);
}

function buildProposalReviewScopeKeys(): DerivedTagActionableSessionScopeKeys {
  const state = getCurrentDerivedTagMigrationAuthoredState();
  const keys = createEmptyScopeKeys();

  for (const [category, assignmentReviews] of Object.entries(state.assignmentReviews) as Array<
    [SearchCategory, { decisions: Array<{ family: string; tag: string; source?: string }> }]
  >) {
    for (const decision of assignmentReviews.decisions) {
      if (decision.source !== "llm") {
        continue;
      }
      addFamily(keys, category, decision.family);
      addTag(keys, category, decision.tag);
    }
  }

  for (const [category, exemplarReviews] of Object.entries(state.exemplarReviews) as Array<
    [SearchCategory, { decisions: Array<{ tag: string; status: string; source?: string }> }]
  >) {
    for (const decision of exemplarReviews.decisions) {
      if (decision.status !== "needs_review" || decision.source !== "llm") {
        continue;
      }
      addTag(keys, category, decision.tag);
    }
  }

  return keys;
}

function buildExemplarCleanupScopeKeys(exemplarLimit: number | undefined): DerivedTagActionableSessionScopeKeys {
  const state = getCurrentDerivedTagMigrationAuthoredState();
  const keys = createEmptyScopeKeys();

  for (const category of DERIVED_TAG_MANAGED_CATEGORIES) {
    for (const exemplarSet of state.exemplars[category].exemplars) {
      const placementCount = (exemplarSet.positives?.length ?? 0) + (exemplarSet.negatives?.length ?? 0);
      if (exemplarLimit !== undefined && placementCount <= exemplarLimit) {
        continue;
      }
      addTag(keys, category, exemplarSet.tag);
    }
  }

  return keys;
}

export function getActionableSessionScopeKeys(
  mode: DerivedTagMigrationMode,
  exemplarLimit: number | undefined,
): DerivedTagActionableSessionScopeKeys | null {
  if (mode === "proposal_review") {
    return buildProposalReviewScopeKeys();
  }
  if (mode === "exemplar_cleanup") {
    return buildExemplarCleanupScopeKeys(exemplarLimit);
  }
  return null;
}
