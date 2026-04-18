import type {
  DerivedTagAuthoredCategoryOntology,
  DerivedTagAuthoredTag,
  DerivedTagCompositeSelector,
  DerivedTagOntologyFamily,
  DerivedTagOntologyTag,
} from "../../types.js";
import { normalizeText } from "../../utils.js";

function normalizeDerivedTag(value: string): string {
  return normalizeText(value).replace(/\s+/g, "_");
}

function pushUnique(target: string[], value: string): void {
  if (!target.includes(value)) {
    target.push(value);
  }
}

export function fromTag(tag: string): DerivedTagCompositeSelector {
  return { kind: "tag", tag };
}

export function fromFamily(
  family: string,
  options: Omit<Extract<DerivedTagCompositeSelector, { kind: "family" }>, "kind" | "family"> = {},
): DerivedTagCompositeSelector {
  return { kind: "family", family, ...options };
}

function resolveCompositeSelectors(
  ontology: DerivedTagAuthoredCategoryOntology,
  tag: DerivedTagAuthoredTag,
  familyEntries: Map<string, DerivedTagAuthoredTag[]>,
): string[] | undefined {
  const resolvedTags: string[] = [];

  for (const childTag of tag.compositeOfAnyTags ?? []) {
    pushUnique(resolvedTags, normalizeDerivedTag(childTag));
  }

  for (const selector of tag.compositeOfAny ?? []) {
    if (selector.kind === "tag") {
      pushUnique(resolvedTags, normalizeDerivedTag(selector.tag));
      continue;
    }

    const normalizedFamily = normalizeDerivedTag(selector.family);
    const familyTags = familyEntries.get(normalizedFamily);
    if (!familyTags || familyTags.length === 0) {
      throw new Error(
        `Derived tag "${normalizeDerivedTag(tag.tag)}" in category "${ontology.category}" references unknown composite family "${normalizedFamily}".`,
      );
    }

    const excludedTags = new Set((selector.excludeTags ?? []).map((value) => normalizeDerivedTag(value)));
    for (const excludedTag of excludedTags) {
      if (!familyTags.some((familyTag) => normalizeDerivedTag(familyTag.tag) === excludedTag)) {
        throw new Error(
          `Derived tag "${normalizeDerivedTag(tag.tag)}" in category "${ontology.category}" excludes unknown family tag "${excludedTag}" from family "${normalizedFamily}".`,
        );
      }
    }

    for (const familyTag of familyTags) {
      const normalizedChildTag = normalizeDerivedTag(familyTag.tag);
      if (selector.include !== "all_tags" && familyTag.assignmentMode === "composite") {
        continue;
      }
      if (excludedTags.has(normalizedChildTag)) {
        continue;
      }
      pushUnique(resolvedTags, normalizedChildTag);
    }
  }

  const normalizedTags = resolvedTags.filter((value) => value !== normalizeDerivedTag(tag.tag));

  if (normalizedTags.length === 0) {
    if (
      (tag.compositeOfAnyTags && tag.compositeOfAnyTags.length > 0) ||
      (tag.compositeOfAny && tag.compositeOfAny.length > 0)
    ) {
      throw new Error(
        `Derived tag "${normalizeDerivedTag(tag.tag)}" in category "${ontology.category}" resolved an empty composite child list.`,
      );
    }
    return undefined;
  }

  return normalizedTags;
}

export function flattenDerivedTagAuthoredCategoryOntology(ontology: DerivedTagAuthoredCategoryOntology): {
  families: DerivedTagOntologyFamily[];
  tags: DerivedTagOntologyTag[];
} {
  const families: DerivedTagOntologyFamily[] = [];
  const tags: DerivedTagOntologyTag[] = [];
  const familyEntries = new Map<string, DerivedTagAuthoredTag[]>();

  for (const [family, definition] of Object.entries(ontology.families)) {
    familyEntries.set(normalizeDerivedTag(family), definition.tags);
    families.push({
      category: ontology.category,
      family,
      axis: definition.axis,
      subcategories: definition.subcategories,
      description: definition.description,
      variantInheritance: definition.variantInheritance,
    });
  }

  for (const [family, definition] of Object.entries(ontology.families)) {
    for (const tag of definition.tags) {
      const compositeOfAnyTags = resolveCompositeSelectors(ontology, tag, familyEntries);
      const publishedTag = { ...tag };
      delete publishedTag.compositeOfAny;
      tags.push({
        category: ontology.category,
        family,
        ...publishedTag,
        ...(compositeOfAnyTags ? { compositeOfAnyTags } : {}),
      });
    }
  }

  return { families, tags };
}
