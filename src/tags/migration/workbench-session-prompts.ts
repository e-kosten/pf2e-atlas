import { DatabaseSync } from "node:sqlite";

import type { SearchCategory, SearchSubcategory } from "../../types.js";
import { normalizeDerivedTag } from "../index.js";
import { parseInteger } from "./cli-utils.js";
import { getActionableSessionScopeKeys } from "./actionable-session-scope.js";
import { summarizeDerivedTagCategoryScopes } from "./category-scope-summary.js";
import {
  compareDisplayText,
  compareManagedCategory,
  DERIVED_TAG_MANAGED_CATEGORIES,
} from "./list-sorting.js";
import {
  getPublishedDerivedTagMigrationOntology,
} from "./runtime-state.js";
import type { DerivedTagTerminalApp, DerivedTagTerminalSelectOption } from "../../tui/terminal-ui.js";
import type { DerivedTagMigrationMode } from "./types.js";

const ANY_CATEGORY = "__all_categories__";
const ANY_SUBCATEGORY = "__all_subcategories__";
const ANY_FAMILY = "__all_families__";
const ANY_TAG = "__all_tags__";

export type DerivedTagMigrationWorkbenchSessionOptions = {
  category?: SearchCategory;
  subcategory?: SearchSubcategory;
  family?: string;
  tag?: string;
  limit?: number;
  exemplarLimit?: number;
};

export function formatDerivedTagMigrationModeLabel(mode: DerivedTagMigrationMode): string {
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

function uniqueSorted(values: string[]): string[] {
  return [...new Set(values)].sort((left, right) => compareDisplayText(left, right) || left.localeCompare(right));
}

function getSessionScopeOntology() {
  return getPublishedDerivedTagMigrationOntology();
}

function buildCategorySelectOptions(
  mode: DerivedTagMigrationMode,
  db: DatabaseSync,
  required: boolean,
): DerivedTagTerminalSelectOption<string>[] {
  const scopeSummary = summarizeDerivedTagCategoryScopes(db, mode);
  const categoryOptions = DERIVED_TAG_MANAGED_CATEGORIES.map((category) => {
    const detailLines = scopeSummary.categories.find((entry) => entry.category === category)?.detailLines ?? [];
    return {
      value: category,
      label: category,
      detailLines: [
        { text: category, tone: "section" },
        ...detailLines.map((line) => ({ text: line })),
      ],
    } satisfies DerivedTagTerminalSelectOption<string>;
  });

  if (required) {
    return categoryOptions;
  }

  return [
    {
      value: ANY_CATEGORY,
      label: "All categories",
      detailLines: [
        { text: "All categories", tone: "section" },
        ...scopeSummary.allCategoriesDetailLines.map((line) => ({ text: line })),
      ],
    },
    ...categoryOptions,
  ];
}

function listSubcategoriesForCategory(category: SearchCategory): SearchSubcategory[] {
  return uniqueSorted(
    getSessionScopeOntology().families
      .filter((family) => family.category === category)
      .flatMap((family) => family.subcategories ?? []),
  ) as SearchSubcategory[];
}

function buildSubcategorySelectOptions(category: SearchCategory): DerivedTagTerminalSelectOption<string>[] {
  const ontology = getSessionScopeOntology();
  const subcategories = listSubcategoriesForCategory(category);
  return [
    {
      value: ANY_SUBCATEGORY,
      label: "All subcategories",
      detailLines: [
        { text: `${category} / all subcategories`, tone: "section" },
        { text: "Keep the session scoped to the full category." },
      ],
    },
    ...subcategories.map((subcategory) => {
      const matchingFamilies = ontology.families.filter((family) =>
        family.category === category && (family.subcategories?.includes(subcategory) ?? false));
      const matchingTags = ontology.tags.filter((tag) => {
        if (tag.category !== category) {
          return false;
        }
        const family = ontology.familyByKey.get(`${tag.category}:${normalizeDerivedTag(tag.family)}` as `${SearchCategory}:${string}`);
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
    }),
  ];
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
  mode: DerivedTagMigrationMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  exemplarLimit: number | undefined,
): DerivedTagTerminalSelectOption<string>[] {
  const ontology = getSessionScopeOntology();
  const actionableScope = getActionableSessionScopeKeys(mode, exemplarLimit);
  const familyOptions = ontology.families
    .filter((family) => familyMatchesScope(category, subcategory, family))
    .filter((family) => !actionableScope || actionableScope.familyKeys.has(`${family.category}:${normalizeDerivedTag(family.family)}` as `${SearchCategory}:${string}`))
    .sort((left, right) =>
      compareManagedCategory(left.category, right.category)
      || compareDisplayText(left.axis, right.axis)
      || compareDisplayText(left.family, right.family)
      || left.family.localeCompare(right.family))
    .map((family) => ({
      value: category ? family.family : `${family.category}:${family.family}`,
      label: category ? `${family.axis} / ${family.family}` : `${family.category} / ${family.axis} / ${family.family}`,
      detailLines: [
        { text: family.family, tone: "section" },
        { text: family.description },
        { text: `Category: ${family.category}` },
        { text: `Axis: ${family.axis}` },
        { text: `Scope: ${family.subcategories?.join(", ") ?? "(all subcategories)"}` },
        { text: `Variant inheritance: ${family.variantInheritance ? "yes" : "no"}` },
      ],
    } satisfies DerivedTagTerminalSelectOption<string>));

  return [
    {
      value: ANY_FAMILY,
      label: "All families",
      detailLines: [
        { text: "All families", tone: "section" },
        { text: "Keep family unspecified and review the wider queue slice." },
      ],
    } satisfies DerivedTagTerminalSelectOption<string>,
    ...familyOptions,
  ];
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

  const family = getSessionScopeOntology().familyByKey.get(`${tag.category}:${normalizeDerivedTag(tag.family)}` as `${SearchCategory}:${string}`);
  if (!family?.subcategories || family.subcategories.length === 0) {
    return true;
  }

  return family.subcategories.includes(subcategory);
}

function buildTagSelectOptions(
  mode: DerivedTagMigrationMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  family: string | undefined,
  exemplarLimit: number | undefined,
  required: boolean,
): DerivedTagTerminalSelectOption<string>[] {
  const ontology = getSessionScopeOntology();
  const actionableScope = getActionableSessionScopeKeys(mode, exemplarLimit);
  const tagOptions = ontology.tags
    .filter((tag) => tagMatchesScope(category, subcategory, family, tag))
    .filter((tag) => !actionableScope || actionableScope.tagKeys.has(`${tag.category}:${normalizeDerivedTag(tag.tag)}` as `${SearchCategory}:${string}`))
    .sort((left, right) =>
      compareManagedCategory(left.category, right.category)
      || compareDisplayText(left.family, right.family)
      || compareDisplayText(left.tag, right.tag)
      || left.tag.localeCompare(right.tag))
    .map((tag) => {
      const resolvedFamily = ontology.familyByKey.get(`${tag.category}:${normalizeDerivedTag(tag.family)}` as `${SearchCategory}:${string}`);
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

  if (required) {
    return tagOptions;
  }

  return [
    {
      value: ANY_TAG,
      label: "All tags",
      detailLines: [
        { text: "All tags", tone: "section" },
        { text: "Keep tag unspecified and create a broader review session." },
      ],
    },
    ...tagOptions,
  ];
}

async function promptCategory(
  terminal: DerivedTagTerminalApp,
  db: DatabaseSync,
  mode: DerivedTagMigrationMode,
  required: boolean,
): Promise<SearchCategory | null | undefined> {
  const value = await terminal.promptSelectOption({
    title: "Session Scope",
    subtitle: "Choose a category boundary for the session",
    prompt: "Categories",
    entries: buildCategorySelectOptions(mode, db, required),
  });

  if (value === undefined) {
    return undefined;
  }
  if (value === ANY_CATEGORY) {
    return null;
  }
  return value as SearchCategory;
}

async function promptSubcategory(
  terminal: DerivedTagTerminalApp,
  category: SearchCategory,
): Promise<SearchSubcategory | null | undefined> {
  const options = buildSubcategorySelectOptions(category);
  if (options.length <= 1) {
    return null;
  }

  const value = await terminal.promptSelectOption({
    title: "Session Scope",
    subtitle: `Optionally narrow ${category} to a subcategory`,
    prompt: "Subcategories",
    entries: options,
  });

  if (value === undefined) {
    return undefined;
  }
  if (value === ANY_SUBCATEGORY) {
    return null;
  }
  return value as SearchSubcategory;
}

async function promptTag(
  terminal: DerivedTagTerminalApp,
  mode: DerivedTagMigrationMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  family: string | undefined,
  exemplarLimit: number | undefined,
  required: boolean,
): Promise<{ category?: SearchCategory; tag?: string } | undefined> {
  const value = await terminal.promptSelectOption({
    title: "Session Scope",
    subtitle: required
      ? "Choose the tag to review"
      : "Optionally narrow the session to one tag",
    prompt: "Tags",
    entries: buildTagSelectOptions(mode, category, subcategory, family, exemplarLimit, required),
  });

  if (value === undefined) {
    return undefined;
  }
  if (value === ANY_TAG) {
    return {};
  }
  if (!category) {
    const [resolvedCategory, resolvedTag] = value.split(":", 2);
    if (resolvedCategory && resolvedTag) {
      return {
        category: resolvedCategory as SearchCategory,
        tag: resolvedTag,
      };
    }
  }
  return { tag: value };
}

async function promptFamily(
  terminal: DerivedTagTerminalApp,
  mode: DerivedTagMigrationMode,
  category: SearchCategory | undefined,
  subcategory: SearchSubcategory | undefined,
  exemplarLimit: number | undefined,
): Promise<{ category?: SearchCategory; family?: string } | undefined> {
  const value = await terminal.promptSelectOption({
    title: "Session Scope",
    subtitle: "Optionally narrow the queue to one ontology family",
    prompt: "Families",
    entries: buildFamilySelectOptions(mode, category, subcategory, exemplarLimit),
  });

  if (value === undefined) {
    return undefined;
  }
  if (value === ANY_FAMILY) {
    return {};
  }
  if (!category) {
    const [resolvedCategory, resolvedFamily] = value.split(":", 2);
    if (resolvedCategory && resolvedFamily) {
      return {
        category: resolvedCategory as SearchCategory,
        family: resolvedFamily,
      };
    }
  }
  return { family: value };
}

async function promptInteger(
  terminal: DerivedTagTerminalApp,
  prompt: string,
  flagName: string,
): Promise<number | undefined> {
  while (true) {
    const value = normalizeOptional(await terminal.promptTextInput({
      title: "Session Scope",
      prompt,
    }));

    try {
      return parseInteger(value, flagName);
    } catch (error) {
      await terminal.pauseForAnyKey((error as Error).message);
    }
  }
}

export async function promptDerivedTagMigrationWorkbenchSessionOptions(
  terminal: DerivedTagTerminalApp,
  db: DatabaseSync,
  mode: DerivedTagMigrationMode,
): Promise<DerivedTagMigrationWorkbenchSessionOptions | undefined> {
  const requireCategory = mode === "legacy_rule";
  const categorySelection = await promptCategory(terminal, db, mode, requireCategory);
  if (categorySelection === undefined) {
    return undefined;
  }
  const category = categorySelection ?? undefined;

  const subcategorySelection = category ? await promptSubcategory(terminal, category) : null;
  if (subcategorySelection === undefined) {
    return undefined;
  }
  const subcategory = subcategorySelection ?? undefined;

  const exemplarLimit = mode === "exemplar_cleanup"
    ? await promptInteger(terminal, "exemplar-limit (blank for none)", "--exemplar-limit")
    : undefined;

  const familySelection = mode === "review_queue" || mode === "proposal_review" || mode === "exemplar_cleanup"
    ? await promptFamily(terminal, mode, category, subcategory, exemplarLimit)
    : {};
  if (familySelection === undefined) {
    return undefined;
  }
  const resolvedCategory = category ?? familySelection.category;
  const family = familySelection.family;

  const tagSelection = await promptTag(
    terminal,
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

  const limit = await promptInteger(terminal, "limit (blank for default)", "--limit");

  return {
    category: resolvedTagCategory,
    subcategory,
    family,
    tag,
    limit,
    exemplarLimit,
  };
}
