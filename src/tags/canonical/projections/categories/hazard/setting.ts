import { projectionFamily, type CategoryProjectionFamilyBlock } from "../../../builders.js";
import { CANONICAL_PROJECTION_FAMILIES } from "../../families.js";

export const hazardSettingProjectionFamilies = [
  projectionFamily(CANONICAL_PROJECTION_FAMILIES.HAZARD.SETTING_SETTING, {
    aquatic_hazard: {
      description:
        "Hazard strongly associated with flooded chambers, rivers, docks, ships, reefs, or underwater spaces.",
    },
    battlefield_hazard: {
      description:
        "Hazard strongly associated with siegeworks, trenches, killing grounds, war engines, or other battlefield scenes.",
    },
    bridge_passage_hazard: {
      description:
        "Hazard strongly associated with bridges, chokepoints, gates, stairwells, or forced passage bottlenecks.",
      appliesWhen: [
        "The hazard is naturally retrieved for narrow crossings, gates, stairwells, or other spaces where intruders must pass through a constrained route.",
        "Forced-passage geometry matters more than a broader dungeon, urban, or wilderness setting identity.",
      ],
      doesNotApplyWhen: [
        "The hazard only happens to sit near a doorway or bridge once without the chokepoint being central to its design.",
        "The stronger fit is threshold_lockdown or forced_movement because the route bottleneck is not the main retrieval hook.",
      ],
      adjacentTags: ["threshold_lockdown", "forced_movement"],
    },
    dungeon_hazard: {
      description:
        "Hazard strongly associated with dungeon corridors, chambers, trapped passages, or underground complexes.",
      appliesWhen: [
        "The hazard is naturally retrieved as a classic corridor, chamber, or trap-complex defense in a dungeon environment.",
        "Its encounter identity is more about underground built-space adventuring than a narrower tomb or temple context.",
      ],
      doesNotApplyWhen: [
        "The hazard is more specifically tomb-, temple-, bridge-, or aquatic-coded than generic dungeon-coded.",
        "The hazard merely happens to appear indoors once without dungeon-like scene identity.",
      ],
      adjacentTags: ["tomb_hazard", "temple_hazard"],
    },
    temple_hazard: {
      description: "Hazard strongly associated with shrines, sanctums, cathedrals, monasteries, or consecrated sites.",
      appliesWhen: [
        "Sacred architecture, ritual sanctums, or faith-site defense is central to the hazard's scene identity.",
        "The hazard is naturally retrieved for shrine, sanctum, or temple protection rather than generic indoor defense.",
      ],
      doesNotApplyWhen: [
        "The hazard is merely magical or haunted without specific sacred-site framing.",
        "The stronger setting is dungeon_hazard or urban_hazard rather than temple-focused.",
      ],
      adjacentTags: ["dungeon_hazard", "appeasement_countermeasure"],
    },
    tomb_hazard: {
      description: "Hazard strongly associated with tombs, barrows, mausoleums, crypts, or burial defenses.",
      appliesWhen: [
        "Burial sites, funerary wards, grave robbing countermeasures, or undead-rest protections are central to the hazard.",
        "A user would retrieve it for crypts, tombs, barrows, or similar funerary scenes.",
      ],
      doesNotApplyWhen: [
        "The hazard is only underground or haunted without real funerary or burial-site identity.",
        "The stronger placement is temple_hazard or generic dungeon_hazard.",
      ],
      adjacentTags: ["dungeon_hazard", "temple_hazard"],
    },
    urban_hazard: {
      description:
        "Hazard strongly associated with civic districts, alleys, rooftops, sewers, shops, or other built urban spaces.",
    },
    wilderness_hazard: {
      description:
        "Hazard strongly associated with forests, trails, camps, ruins in the wild, or other outdoor expedition scenes.",
    },
  }),
] satisfies CategoryProjectionFamilyBlock<"hazard">[];
