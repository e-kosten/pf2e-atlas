import { defineFacetConcepts, mergeCanonicalConceptSeeds, type FacetlessConceptSeed } from "../../builders.js";
import { CANONICAL_FACETS } from "../../facets.js";

const habitatSeeds: Record<string, FacetlessConceptSeed> = {
  aquatic_setting: {},
  arctic_setting: {},
  canyon_setting: {},
  coastal_setting: {},
  desert_setting: {},
  forest_setting: {},
  freshwater_setting: {},
  island_setting: {},
  jungle_setting: {},
  mountain_setting: {},
  plains_setting: {},
  sky_setting: {},
  swamp_setting: {},
  underground_setting: {},
  volcanic_setting: {},
  wasteland_setting: {},
};

const planarSeeds: Record<string, FacetlessConceptSeed> = {
  abaddon_setting: {},
  abyss_setting: {},
  astral_setting: {},
  axis_setting: {},
  boneyard_setting: {},
  dreamlands_setting: {},
  elysium_setting: {},
  ethereal_setting: {},
  first_world_setting: {},
  heaven_setting: {},
  hell_setting: {},
  maelstrom_setting: {},
  nirvana_setting: {},
  plane_of_air_setting: {},
  plane_of_earth_setting: {},
  plane_of_fire_setting: {},
  plane_of_water_setting: {},
  shadow_plane_setting: {},
};

const regionalSeeds: Record<string, FacetlessConceptSeed> = {
  alien_technology_wasteland_setting: {
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  darklands_setting: {
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  demonic_scar_frontier_setting: {
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  gothic_horror_land_setting: {
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  magic_blight_wasteland_setting: {
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  mwangi_setting: {
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  organized_undead_society_setting: {
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  tian_xia_setting: {
    text: {
      definition: "Approved primary setting shape.",
    },
  },
  undead_war_torn_region_setting: {
    text: {
      definition: "Approved primary setting shape.",
    },
  },
};

const siteSeeds: Record<string, FacetlessConceptSeed> = {
  aquatic_hazard: {},
  battlefield_hazard: {},
  battlefield_setting: {},
  bridge_passage_hazard: {},
  dungeon_hazard: {},
  fortress_setting: {},
  graveyard_setting: {},
  nautical_setting: {},
  ruins_setting: {},
  rural_setting: {},
  small_settlement_setting: {},
  temple_hazard: {},
  temple_setting: {},
  tomb_hazard: {},
  urban_hazard: {},
  urban_setting: {},
  wilderness_hazard: {},
};

export const settingSeedsByKind = mergeCanonicalConceptSeeds([
  defineFacetConcepts(
    CANONICAL_FACETS.SETTING.HABITAT,
    habitatSeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.SETTING.PLANAR,
    planarSeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.SETTING.REGIONAL,
    regionalSeeds,
  ),
  defineFacetConcepts(
    CANONICAL_FACETS.SETTING.SITE,
    siteSeeds,
  ),
]);
