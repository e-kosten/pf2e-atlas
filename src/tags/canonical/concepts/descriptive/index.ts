import { CANONICAL_VOCABULARY } from "../../vocabulary.js";
import { buildCanonicalConceptMap, mergeCanonicalConceptSeeds, type CanonicalConceptSeed } from "../../builders.js";

import { behaviorOverrideSeedsByKind } from "./behavior_override.js";
import { capabilitySeedsByKind } from "./capability.js";
import { challengeStructureSeedsByKind } from "./challenge_structure.js";
import { creatureFamilySeedsByKind } from "./creature_family.js";
import { deliverySeedsByKind } from "./delivery.js";
import { effectSeedsByKind } from "./effect.js";
import { functionSeedsByKind } from "./function.js";
import { mechanismSeedsByKind } from "./mechanism.js";
import { pathogenesisSeedsByKind } from "./pathogenesis.js";
import { progressionSeedsByKind } from "./progression.js";
import { responseDemandSeedsByKind } from "./response_demand.js";
import { roleSeedsByKind } from "./role.js";
import { settingSeedsByKind } from "./setting.js";
import { themeSeedsByKind } from "./theme.js";

const descriptiveConceptSeeds: Record<string, CanonicalConceptSeed> =
  mergeCanonicalConceptSeeds([
    behaviorOverrideSeedsByKind,
    capabilitySeedsByKind,
    challengeStructureSeedsByKind,
    creatureFamilySeedsByKind,
    deliverySeedsByKind,
    effectSeedsByKind,
    functionSeedsByKind,
    mechanismSeedsByKind,
    pathogenesisSeedsByKind,
    progressionSeedsByKind,
    responseDemandSeedsByKind,
    roleSeedsByKind,
    settingSeedsByKind,
    themeSeedsByKind,
  ]);

export const DERIVED_TAG_DESCRIPTIVE_CANONICAL_CONCEPTS_BY_ID = buildCanonicalConceptMap(
  CANONICAL_VOCABULARY.SCHEMA.KIND.DESCRIPTIVE,
  descriptiveConceptSeeds,
);
