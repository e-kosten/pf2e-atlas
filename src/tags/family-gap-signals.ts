import { normalizeDerivedTag } from "./shared.js";

export type FamilyGapFeatureBucket =
  | "possible_place_anchor"
  | "context_only"
  | "taxonomy"
  | "variant_marker"
  | "unknown";

export type FamilyGapFeatureClassification = {
  bucket: FamilyGapFeatureBucket;
  eligibleForClustering: boolean;
  familyConceptWeight: number;
  existingTagOverlaps: string[];
  primaryCue: string | null;
  suppressionReason: string | null;
};

type GenericFeatureKind =
  | "nameToken"
  | "namePhrase"
  | "descriptionToken"
  | "descriptionPhrase"
  | "trait"
  | "reference"
  | "name"
  | "name_phrase"
  | "text"
  | "text_phrase"
  | "target"
  | "scope";

const VARIANT_MARKERS = new Set([
  "adult",
  "ancient",
  "young",
  "spellcaster",
  "bb",
  "level",
  "elite",
]);

const TAXONOMY_TERMS = new Set([
  "aberration",
  "animal",
  "beast",
  "celestial",
  "construct",
  "dragon",
  "elemental",
  "fiend",
  "humanoid",
  "monitor",
  "ooze",
  "spirit",
  "undead",
]);

const CONTEXT_ONLY_TERMS = [
  "dwells",
  "dwells in",
  "found in",
  "haunts",
  "haunts the",
  "lair",
  "lairs",
  "lurks",
  "lurks in",
  "native to",
];

const EXISTING_SETTING_TAG_CUES: Record<string, string[]> = {
  aquatic_setting: ["aquatic", "underwater", "sea", "ocean", "marine"],
  freshwater_setting: ["freshwater", "river", "lake", "pond", "stream", "spring"],
  coastal_setting: ["coast", "coastal", "shore", "reef", "littoral", "tidepool"],
  astral_setting: ["astral", "silver void"],
  first_world_setting: ["first world", "fey realm"],
  dreamlands_setting: ["dreamlands", "dreaming plane", "leng", "nightgaunt", "shantak"],
  boneyard_setting: ["boneyard", "psychopomp"],
  island_setting: ["island", "archipelago", "isle", "atoll"],
  nautical_setting: ["nautical", "ship", "harbor", "dock", "wreck", "sailor", "pirate"],
  forest_setting: ["forest", "grove", "jungle", "rainforest", "woodland", "canopy"],
  plains_setting: ["plains", "grassland", "prairie", "savanna", "steppe"],
  canyon_setting: ["canyon", "gorge", "mesa", "badlands"],
  swamp_setting: ["swamp", "bog", "marsh", "fen", "mire", "bayou", "mangrove"],
  underground_setting: ["underground", "subterranean", "cave", "cavern", "tunnel", "crypt", "sewer", "mine"],
  urban_setting: ["urban", "city", "street", "alley", "town", "metropolis"],
  arctic_setting: ["arctic", "tundra", "glacier", "icebound", "frozen", "snow"],
  desert_setting: ["desert", "dune", "arid", "sand"],
  wasteland_setting: ["wasteland", "wastes", "barren", "blasted"],
  mountain_setting: ["mountain", "peak", "cliff", "ridge", "foothills", "highlands"],
  graveyard_setting: ["graveyard", "cemetery", "mausoleum", "barrow", "cairn", "burial ground", "tomb"],
  ruins_setting: ["ruin", "ruins", "derelict", "collapsed", "crumbling", "lost city"],
  temple_setting: ["temple", "shrine", "cathedral", "monastery", "chapel", "sanctuary", "abbey", "holy site"],
  fortress_setting: ["fortress", "citadel", "keep", "stronghold", "bastion", "watchtower", "garrison", "castle"],
  volcanic_setting: ["volcanic", "volcano", "lava", "magma", "caldera", "ash"],
  rural_setting: ["rural", "village", "hamlet", "farm", "pasture", "countryside"],
};

const NOVEL_SETTING_CUES = [
  "abyss",
  "abyssal",
  "darklands",
  "erebus",
  "hell",
  "infernal",
  "maelstrom",
  "orv",
  "penumbral",
  "plane of shadow",
  "shadow plane",
  "stygia",
  "umbral",
];

function normalizeSurface(value: string): string {
  return value
    .replace(/^target:/, "")
    .replace(/^scope:/, "")
    .replace(/_/g, " ")
    .trim()
    .toLowerCase();
}

function includesAny(surface: string, candidates: Iterable<string>): boolean {
  for (const candidate of candidates) {
    if (!candidate) {
      continue;
    }
    if (surface.includes(candidate)) {
      return true;
    }
  }
  return false;
}

function findLongestCue(surface: string, candidates: Iterable<string>): string | null {
  let match: string | null = null;
  for (const candidate of candidates) {
    if (!candidate || !surface.includes(candidate)) {
      continue;
    }
    if (!match || candidate.length > match.length) {
      match = candidate;
    }
  }
  return match;
}

function deriveExistingSettingOverlaps(surface: string): string[] {
  const overlaps = new Set<string>();
  for (const [tag, cues] of Object.entries(EXISTING_SETTING_TAG_CUES)) {
    if (includesAny(surface, cues)) {
      overlaps.add(normalizeDerivedTag(tag));
    }
  }
  return [...overlaps].sort();
}

function derivePrimarySettingCue(surface: string, existingTagOverlaps: string[]): string | null {
  const novelCue = findLongestCue(surface, NOVEL_SETTING_CUES);
  if (novelCue) {
    return novelCue;
  }

  const existingCues = existingTagOverlaps.flatMap((tag) => EXISTING_SETTING_TAG_CUES[tag] ?? []);
  return findLongestCue(surface, existingCues);
}

export function classifyFamilyGapFeature(
  family: string | undefined,
  kind: GenericFeatureKind,
  value: string,
): FamilyGapFeatureClassification {
  const normalizedFamily = family ? normalizeDerivedTag(family) : undefined;
  const surface = normalizeSurface(value);

  if (!normalizedFamily || normalizedFamily !== "setting") {
    return {
      bucket: "unknown",
      eligibleForClustering: true,
      familyConceptWeight: 1,
      existingTagOverlaps: [],
      primaryCue: null,
      suppressionReason: null,
    };
  }

  if (kind === "scope") {
    return {
      bucket: "taxonomy",
      eligibleForClustering: false,
      familyConceptWeight: 0.2,
      existingTagOverlaps: [],
      primaryCue: null,
      suppressionReason: "scope",
    };
  }

  if (
    VARIANT_MARKERS.has(surface)
    || /\blevel\b/.test(surface)
    || /\b(?:adult|ancient|young|spellcaster|elite)\b/.test(surface)
  ) {
    return {
      bucket: "variant_marker",
      eligibleForClustering: false,
      familyConceptWeight: 0.08,
      existingTagOverlaps: [],
      primaryCue: null,
      suppressionReason: "variant-marker",
    };
  }

  if ((kind === "trait" && TAXONOMY_TERMS.has(surface)) || TAXONOMY_TERMS.has(surface)) {
    return {
      bucket: "taxonomy",
      eligibleForClustering: false,
      familyConceptWeight: 0.18,
      existingTagOverlaps: [],
      primaryCue: null,
      suppressionReason: "taxonomy",
    };
  }

  const existingTagOverlaps = deriveExistingSettingOverlaps(surface);
  const isNovelCue = includesAny(surface, NOVEL_SETTING_CUES);
  const isPlaceAnchor = existingTagOverlaps.length > 0 || isNovelCue;
  const primaryCue = derivePrimarySettingCue(surface, existingTagOverlaps);

  if (isPlaceAnchor) {
    return {
      bucket: "possible_place_anchor",
      eligibleForClustering: true,
      familyConceptWeight: isNovelCue && existingTagOverlaps.length === 0 ? 1.8 : 1.25,
      existingTagOverlaps,
      primaryCue,
      suppressionReason: null,
    };
  }

  if (includesAny(surface, CONTEXT_ONLY_TERMS)) {
    return {
      bucket: "context_only",
      eligibleForClustering: false,
      familyConceptWeight: 0.35,
      existingTagOverlaps: [],
      primaryCue: null,
      suppressionReason: "context-only",
    };
  }

  return {
    bucket: "unknown",
    eligibleForClustering: false,
    familyConceptWeight: 0.55,
    existingTagOverlaps: [],
    primaryCue: null,
    suppressionReason: "not-setting-salient",
  };
}
