import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const corruptionSeeds: Record<string, FacetlessConceptSeed> = {
  blight_tainted: {},
  cursewarped: {},
  fungal_infested: {},
  nightmare_tainted: {},
  parasite_ridden: {},
  plaguebearing: {},
  void_tainted: {},
};

const hauntManifestationSeeds: Record<string, FacetlessConceptSeed> = {
  battlefield_disruption: {},
  judgment_haunt: {},
  life_drain_hazard: {},
  lure_compulsion: {},
  phantom_assailants: {},
  possession_haunt: {},
  replayed_tragedy: {},
};

const pestilenceSeeds: Record<string, FacetlessConceptSeed> = {
  epidemic_pestilence: {
    text: {
      definition:
        "Heterogeneous browse family; rows split across response_demand, delivery, and theme. Thematic rather than pure response-demand descriptor.",
    },
  },
};

const themeValueSeeds: Record<string, FacetlessConceptSeed> = {
  ancestral_legacy: {},
  apocalypse_ruin: {},
  body_horror: {},
  carnival_show: {},
  corrupted_sacred: {},
  cosmic_dread: {},
  courtly_pageantry: {},
  cursed_transformation: {},
  decadence_decline: {},
  disguised_pretender: {},
  dream_nightmare: {},
  faceless_horror: {},
  folk_horror: {},
  forbidden_knowledge: {},
  funerary_mourning: {},
  industrial_grotesque: {},
  innocence_twisted: {},
  living_artwork: {},
  living_toy: {},
  maritime_superstition: {},
  mask_motif: {},
  mirror_motif: {},
  obsession_fixation: {},
  occult_conspiracy: {},
  paranoia_surveillance: {},
  predatory_seduction: {},
  prophecy_omen: {},
  revelry_excess: {},
  ritual_ceremony: {},
  seasonal_festival: {},
  seductive_temptation: {},
  stitched_horror: {},
  trickster_mischief: {},
  vengeful_tragedy: {},
};

export const themeSeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.THEME.CORRUPTION,
    corruptionSeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.THEME.HAUNT_MANIFESTATION,
    hauntManifestationSeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.THEME.PESTILENCE,
    pestilenceSeeds,
  ),
  defineFacetConcepts(CANONICAL_FACETS.THEME.THEME, themeValueSeeds),
]);
