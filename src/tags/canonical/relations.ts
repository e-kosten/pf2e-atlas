import type { DerivedTagCanonicalConceptRelation } from "../../domain/derived-tag-types.js";

export const DERIVED_TAG_CANONICAL_RELATIONS: DerivedTagCanonicalConceptRelation[] = 
[
  {
    fromConceptId: "curse_remediation",
    relation: "counteracts",
    toConceptId: "curse_application"
  },
  {
    fromConceptId: "disease_remediation",
    relation: "counteracts",
    toConceptId: "disease_application"
  },
  {
    fromConceptId: "petrification_remediation",
    relation: "counteracts",
    toConceptId: "petrification_application"
  },
  {
    fromConceptId: "poison_remediation",
    relation: "counteracts",
    toConceptId: "poison_application"
  }
];
