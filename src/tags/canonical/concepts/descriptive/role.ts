import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const roleSeeds: Record<string, FacetlessConceptSeed> = {
  artisan_npc: {},
  authority_npc: {},
  captive_npc: {},
  civic_npc: {},
  court_entourage: {},
  crew_member: {},
  criminal_cell: {},
  criminal_npc: {},
  cult_member: {},
  enforcer_npc: {},
  escort_npc: {},
  guardian_npc: {},
  guardian_retinue: {},
  guide_npc: {},
  healer_npc: {},
  infestation_member: {},
  infiltrator_npc: {},
  merchant_npc: {},
  pack_hunter: {},
  patrol_member: {},
  performer_npc: {},
  profession_npc: {},
  religious_npc: {},
  scholar_npc: {},
  warband_member: {},
  watcher_npc: {},
};

const tacticalRoleSeeds: Record<string, FacetlessConceptSeed> = {
  ambusher_combatant: {},
  artillery_combatant: {},
  brute_combatant: {},
  commander_combatant: {},
  controller_combatant: {},
  defender_combatant: {},
  harrier_combatant: {},
  skirmisher_combatant: {},
  support_combatant: {},
};

export const roleSeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(CANONICAL_FACETS.ROLE.ROLE, roleSeeds),
  defineFacetConcepts(
    CANONICAL_FACETS.ROLE.TACTICAL,
    tacticalRoleSeeds,
  ),
]);
