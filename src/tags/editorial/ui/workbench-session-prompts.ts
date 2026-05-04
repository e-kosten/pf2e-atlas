import { DatabaseSync } from "node:sqlite";

import { normalizeSearchCategory } from "../../../domain/categories.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/derived-tag-types.js";
import type { TerminalInteractionContextAdapters } from "../../../tui/interaction-context-adapters.js";
import { normalizeDerivedTag } from "../../runtime/matcher/shared.js";
import { getActionableSessionScopeKeys } from "../sessions/actionable-session-scope.js";
import { summarizeDerivedTagCategoryScopes } from "../sessions/category-scope-summary.js";
import { compareDisplayText, compareManagedCategory, DERIVED_TAG_MANAGED_CATEGORIES } from "../list-sorting.js";
import { getPublishedDerivedTagOntology } from "../state/runtime-state.js";
import type { DerivedTagTerminalSelectOption } from "../../../tui/terminal-ui.js";
import type { DerivedTagWorkbenchMode } from "../types.js";

export type DerivedTagWorkbenchSessionOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  family?: string;
  tag?: string;
  limit?: number;
  exemplarLimit?: number;
};

export type DerivedTagWorkbenchPromptAdapters = Pick<
  TerminalInteractionContextAdapters,
  "promptOptionalSelectOption" | "promptSelectOption" | "promptTextInput"
>;

export type DerivedTagWorkbenchSessionPrompts = DerivedTagWorkbenchPromptAdapters & {
  pauseForAnyKey: (message: string) => Promise<void>;
};

function buildOntologyKey(category: SearchCategory, value: string): `${SearchCategory}:${string}` {
  return `${category}:${normalizeDerivedTag(value)}`;
}

export function formatDerivedTagWorkbenchModeLabel(mode: DerivedTagWorkbenchMode): string {
  if (mode === "proposal_review") {
    return "AI proposal review";
  }
  if (mode === "review_queue") {
    return "review queue";
  }
  return mode.replaceAll("_", " ");
}

function normalizeOptional(value: string | undefined): string | undefined {
  const trimmed = value?.trim();
  return trimmed ? trimmed : undefined;
}

function parseOptionalInteger(value: string | undefined, flagName: string): number | undefined {
  if (value === undefined) {
    return undefined;
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) {
    throw new Error(`Expected ${flagName} to be an integer, received "${value}".`);
  }
  return parsed;
}

function uniqueSorted<T extends string>(values: T[]): T[] {
  return [...new Set(values)].sort((left, right) => compareDisplayText(left, right) || left.localeCompare(right));
}

function getSessionScopeOntology() {
  return getPublishedDerivedTagOntology();
}

function buildCategorySelectOptions(
  mode: DerivedTagWorkbenchMode,
  db: DatabaseSync,
): DerivedTagTerminalSelectOption<SearchCategory>[] {
  const scopeSummary = summarizeDerivedTagCategoryScopes(db, mode);
  return DERIVED_TAG_MANAGED_CATEGORIES.map((category) => {
    const detailLines = scopeSummary.categories.find((entry) => entry.category === category)?.detailLines ?? [];
    return {
      value: category,
      label: category,
      detailLines: [{ text: category, tone: "section" }, ...detailLines.map((line) => ({ text: line }))],
    } satisfies DerivedTagTerminalSelectOption<string>;
  });
}

function buildAllCategoryOption(
  mode: DerivedTagWorkbenchMode,
  db: DatabaseSync,
): Pick<DerivedTagTerminalSelectOption<string>, "label" | "description" | "detailLines"> {
  const scopeSummary = summarizeDerivedTagCategoryScopes(db, mode);
  return {
    label: "All categories",
    detailLines: [
      { text: "All categories", tone: "section" },
      ...scopeSummary.allCategoriesDetailLines.map((line) => ({ text: line })),
    ],
  };
}

function listSubcategoriesForCategory(category: SearchCategory): SearchSubcategory[] {
  const subcategories = getSessionScopeOntology()
    .families.filter((family) => family.category === category)
    .reduce<SearchSubcategory[]>(
      (allSubcategories, family) => [...allSubcategories, ...(family.subcategories ?? [])],
      [],
    );
  return uniqueSorted(subcategories);
}

function buildSubcategorySelectOptions(category: SearchCategory): DerivedTagTerminalSelectOption<SearchSubcategory>[] {
  const ontology = getSessionScopeOntology();
  const subcategories = listSubcategoriesForCategory(category);
  return subcategories.map((subcategory) => {
    const matchingFamilies = ontology.families.filter(
      (family) => family.category === category && (family.subcategories?.includes(subcategory) ?? false),
    );
    const matchingTags = ontology.tags.filter((tag) => {
      if (tag.category !== category) {
        return false;
      }
      const family = ontology.familyByKey.get(buildOntologyKey(tag.category, tag.family));
      return family?.subcategories?.includes(subcategory) ?? false;
    });
    return {
      value: subcategory,
      label: subcategory,
      detailLines: [
        { text: `${category}/${subcategory}`, tone: "section" },
        { text: `${matchingFamilies.length} families apply` },
        { text: `${matchingTags.length} tags apply` },
      ],
    } satisfies DerivedTagTerminalSelectOption<string>;
  });
}

function buildAllSubcategoryOption(
  category: SearchCategory,
): Pick<DerivedTagTerminalSelectOption<string>, "label" | "description" | "detailLines"> {
  return {
    label: "All subcategories",
    detailLines: [
      { text: `${category} / all subcategories`, tone: "section" },
      { text: "Keep the session scoped to the full category." },
    ],
  };
}

function familyMatchesScope(
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  family: { category: SearchCategory; family: string; subcategories?: SearchSubcategory[]; axis: string },
): boolean {
  if (category && family.category !== category) {
    return false;
  }
  if (!subcategory) {
    return true;
  }
  if (!family.subcategories || family.subcategories.length === 0) {
    return true;
  }
  return family.subcategories.includes(subcategory);
}

function buildFamilySelectOptions(
  mode: DerivedTagWorkbenchMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  exemplarLimit: number | undefined,
): DerivedTagTerminalSelectOption<string>[] {
  const ontology = getSessionScopeOntology();
  const actionableScope = getActionableSessionScopeKeys(mode, exemplarLimit);
  const familyOptions = ontology.families
    .filter((family) => familyMatchesScope(category, subcategory, family))
    .filter(
      (family) =>
        !actionableScope ||
        actionableScope.familyKeys.has(
          `${family.category}:${normalizeDerivedTag(family.family)}` as `${SearchCategory}:${string}`,
        ),
    )
    .sort(
      (left, right) =>
        compareManagedCategory(left.category, right.category) ||
        compareDisplayText(left.axis, right.axis) ||
        compareDisplayText(left.family, right.family) ||
        left.family.localeCompare(right.family),
    )
    .map(
      (family) =>
        ({
          value: category ? family.family : `${family.category}:${family.family}`,
          label: category
            ? `${family.axis} / ${family.family}`
            : `${family.category} / ${family.axis} / ${family.family}`,
          detailLines: [
            { text: family.family, tone: "section" },
            { text: family.description },
            { text: `Category: ${family.category}` },
            { text: `Axis: ${family.axis}` },
            { text: `Scope: ${family.subcategories?.join(", ") ?? "(all subcategories)"}` },
            { text: `Variant inheritance: ${family.variantInheritance ? "yes" : "no"}` },
          ],
        }) satisfies DerivedTagTerminalSelectOption<string>,
    );

  return familyOptions;
}

function buildAllFamilyOption(): Pick<DerivedTagTerminalSelectOption<string>, "label" | "description" | "detailLines"> {
  return {
    label: "All families",
    detailLines: [
      { text: "All families", tone: "section" },
      { text: "Keep family unspecified and review the wider queue slice." },
    ],
  };
}

function tagMatchesScope(
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  familyKey: string | undefined,
  tag: { category: SearchCategory; family: string },
): boolean {
  if (category && tag.category !== category) {
    return false;
  }
  if (familyKey && normalizeDerivedTag(tag.family) !== normalizeDerivedTag(familyKey)) {
    return false;
  }
  if (!subcategory) {
    return true;
  }

  const family = getSessionScopeOntology().familyByKey.get(buildOntologyKey(tag.category, tag.family));
  if (!family?.subcategories || family.subcategories.length === 0) {
    return true;
  }

  return family.subcategories.includes(subcategory);
}

function buildTagSelectOptions(
  mode: DerivedTagWorkbenchMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  family: string | undefined,
  exemplarLimit: number | undefined,
): DerivedTagTerminalSelectOption<string>[] {
  const ontology = getSessionScopeOntology();
  const actionableScope = getActionableSessionScopeKeys(mode, exemplarLimit);
  return ontology.tags
    .filter((tag) => tagMatchesScope(category, subcategory, family, tag))
    .filter((tag) => !actionableScope || actionableScope.tagKeys.has(buildOntologyKey(tag.category, tag.tag)))
    .sort(
      (left, right) =>
        compareManagedCategory(left.category, right.category) ||
        compareDisplayText(left.family, right.family) ||
        compareDisplayText(left.tag, right.tag) ||
        left.tag.localeCompare(right.tag),
    )
    .map((tag) => {
      const resolvedFamily = ontology.familyByKey.get(buildOntologyKey(tag.category, tag.family));
      return {
        value: category ? tag.tag : `${tag.category}:${tag.tag}`,
        label: category ? `${tag.family} / ${tag.tag}` : `${tag.category} / ${tag.family} / ${tag.tag}`,
        detailLines: [
          { text: tag.tag, tone: "section" },
          { text: tag.description },
          { text: `Category: ${tag.category}` },
          { text: `Family: ${tag.family}` },
          { text: `Axis: ${resolvedFamily?.axis ?? "(unknown)"}` },
          { text: `Scope: ${resolvedFamily?.subcategories?.join(", ") ?? "(all subcategories)"}` },
          { text: `Assignment mode: ${tag.assignmentMode}` },
        ],
      } satisfies DerivedTagTerminalSelectOption<string>;
    });
}

function buildAllTagOption(): Pick<DerivedTagTerminalSelectOption<string>, "label" | "description" | "detailLines"> {
  return {
    label: "All tags",
    detailLines: [
      { text: "All tags", tone: "section" },
      { text: "Keep tag unspecified and create a broader review session." },
    ],
  };
}

async function promptCategory(
  prompts: DerivedTagWorkbenchPromptAdapters,
  db: DatabaseSync,
  mode: DerivedTagWorkbenchMode,
  required: boolean,
): Promise<SearchCategory | null | undefined> {
  const entries = buildCategorySelectOptions(mode, db);
  if (required) {
    const result = await prompts.promptSelectOption({
      title: "Session Scope",
      subtitle: "Choose a category boundary for the session",
      prompt: "Categories",
      entries,
    });
    return result.kind === "selected" ? result.value : undefined;
  }

  const result = await prompts.promptOptionalSelectOption({
    title: "Session Scope",
    subtitle: "Choose a category boundary for the session",
    prompt: "Categories",
    allOption: buildAllCategoryOption(mode, db),
    entries,
  });

  if (result.kind === "cancelled" || result.kind === "back") {
    return undefined;
  }
  return result.kind === "all" ? null : result.value;
}

async function promptSubcategory(
  prompts: DerivedTagWorkbenchPromptAdapters,
  category: SearchCategory,
): Promise<SearchSubcategory | null | undefined> {
  const options = buildSubcategorySelectOptions(category);
  if (options.length === 0) {
    return null;
  }

  const result = await prompts.promptOptionalSelectOption({
    title: "Session Scope",
    subtitle: `Optionally narrow ${category} to a subcategory`,
    prompt: "Subcategories",
    entries: options,
    allOption: buildAllSubcategoryOption(category),
  });

  if (result.kind === "cancelled" || result.kind === "back") {
    return undefined;
  }
  return result.kind === "all" ? null : result.value;
}

async function promptTag(
  prompts: DerivedTagWorkbenchPromptAdapters,
  mode: DerivedTagWorkbenchMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  family: string | undefined,
  exemplarLimit: number | undefined,
  required: boolean,
): Promise<{ category?: SearchCategory; tag?: string } | undefined> {
  const entries = buildTagSelectOptions(mode, category, subcategory, family, exemplarLimit);
  const result = required
    ? await prompts.promptSelectOption({
        title: "Session Scope",
        subtitle: "Choose the tag to review",
        prompt: "Tags",
        entries,
      })
    : await prompts.promptOptionalSelectOption({
        title: "Session Scope",
        subtitle: "Optionally narrow the session to one tag",
        prompt: "Tags",
        entries,
        allOption: buildAllTagOption(),
      });

  if (result.kind === "cancelled" || result.kind === "back") {
    return undefined;
  }
  if (result.kind === "all") {
    return {};
  }
  const value = result.value;
  if (!category) {
    const [resolvedCategory, resolvedTag] = value.split(":", 2);
    const normalizedCategory = normalizeSearchCategory(resolvedCategory);
    if (normalizedCategory && resolvedTag) {
      return {
        category: normalizedCategory,
        tag: resolvedTag,
      };
    }
  }
  return { tag: value };
}

async function promptFamily(
  prompts: DerivedTagWorkbenchPromptAdapters,
  mode: DerivedTagWorkbenchMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  exemplarLimit: number | undefined,
): Promise<{ category?: SearchCategory; family?: string } | undefined> {
  const result = await prompts.promptOptionalSelectOption({
    title: "Session Scope",
    subtitle: "Optionally narrow the queue to one ontology family",
    prompt: "Families",
    entries: buildFamilySelectOptions(mode, category, subcategory, exemplarLimit),
    allOption: buildAllFamilyOption(),
  });

  if (result.kind === "cancelled" || result.kind === "back") {
    return undefined;
  }
  if (result.kind === "all") {
    return {};
  }
  const value = result.value;
  if (!category) {
    const [resolvedCategory, resolvedFamily] = value.split(":", 2);
    const normalizedCategory = normalizeSearchCategory(resolvedCategory);
    if (normalizedCategory && resolvedFamily) {
      return {
        category: normalizedCategory,
        family: resolvedFamily,
      };
    }
  }
  return { family: value };
}

async function promptInteger(
  prompts: DerivedTagWorkbenchSessionPrompts,
  prompt: string,
  flagName: string,
): Promise<number | undefined> {
  for (;;) {
    const value = normalizeOptional(
      await prompts.promptTextInput({
        title: "Session Scope",
        prompt,
      }),
    );

    try {
      return parseOptionalInteger(value, flagName);
    } catch (error) {
      await prompts.pauseForAnyKey((error as Error).message);
    }
  }
}

export async function promptDerivedTagWorkbenchSessionOptions(
  prompts: DerivedTagWorkbenchSessionPrompts,
  db: DatabaseSync,
  mode: DerivedTagWorkbenchMode,
): Promise<DerivedTagWorkbenchSessionOptions | undefined> {
  const requireCategory = mode === "legacy_rule";
  const categorySelection = await promptCategory(prompts, db, mode, requireCategory);
  if (categorySelection === undefined) {
    return undefined;
  }
  const category = categorySelection ?? undefined;

  const subcategorySelection = category ? await promptSubcategory(prompts, category) : null;
  if (subcategorySelection === undefined) {
    return undefined;
  }
  const subcategory = subcategorySelection ?? undefined;

  const exemplarLimit =
    mode === "exemplar_cleanup"
      ? await promptInteger(prompts, "exemplar-limit (blank for none)", "--exemplar-limit")
      : undefined;

  const familySelection =
    mode === "review_queue" || mode === "proposal_review" || mode === "exemplar_cleanup"
      ? await promptFamily(prompts, mode, category, subcategory, exemplarLimit)
      : {};
  if (familySelection === undefined) {
    return undefined;
  }
  const resolvedCategory = category ?? familySelection.category;
  const family = familySelection.family;

  const tagSelection = await promptTag(
    prompts,
    mode,
    resolvedCategory,
    subcategory,
    family,
    exemplarLimit,
    mode === "legacy_rule",
  );
  if (tagSelection === undefined) {
    return undefined;
  }
  const resolvedTagCategory = resolvedCategory ?? tagSelection.category;
  const tag = tagSelection.tag;

  const limit = await promptInteger(prompts, "limit (blank for default)", "--limit");

  return {
    category: resolvedTagCategory,
    subcategory,
    family,
    tag,
    limit,
    exemplarLimit,
  };
}
