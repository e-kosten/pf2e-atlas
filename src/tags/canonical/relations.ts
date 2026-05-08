import type { DerivedTagCanonicalConceptRelation } from "../../domain/derived-tag-types.js";
import { relate } from "./builders.js";

export const DERIVED_TAG_CANONICAL_RELATIONS: DerivedTagCanonicalConceptRelation[] = 
[
  relate("curse_remediation", "counteracts", "curse_application"),
  relate("disease_remediation", "counteracts", "disease_application"),
  relate("petrification_remediation", "counteracts", "petrification_application"),
  relate("poison_remediation", "counteracts", "poison_application"),
];
