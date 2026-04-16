import type {
  DerivedTagAuthoredCategoryOntology,
  DerivedTagOntologyFamily,
  DerivedTagOntologyTag,
} from "../../types.js";

export function flattenDerivedTagAuthoredCategoryOntology(
  ontology: DerivedTagAuthoredCategoryOntology,
): { families: DerivedTagOntologyFamily[]; tags: DerivedTagOntologyTag[] } {
  const families: DerivedTagOntologyFamily[] = [];
  const tags: DerivedTagOntologyTag[] = [];

  for (const [family, definition] of Object.entries(ontology.families)) {
    families.push({
      category: ontology.category,
      family,
      subcategories: definition.subcategories,
      description: definition.description,
      variantInheritance: definition.variantInheritance,
    });

    for (const tag of definition.tags) {
      tags.push({
        category: ontology.category,
        family,
        ...tag,
      });
    }
  }

  return { families, tags };
}
