import { normalizeDerivedTag } from "../runtime/shared.js";

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
  cueStrength: "strong" | "weak" | "novel" | "none";
  cueLocality: number;
  cueAmbiguityPenalty: number;
  boilerplateRisk: number;
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

type SettingCueSpec = {
  strong: string[];
  weak?: string[];
};

type SettingCueMatch = {
  tag: string | null;
  cue: string;
  strength: "strong" | "weak" | "novel";
};

const EXISTING_SETTING_TAG_CUES: Record<string, SettingCueSpec> = {
  aquatic_setting: {
    strong: ["aquatic", "underwater", "undersea"],
    weak: ["sea", "ocean", "marine"],
  },
  freshwater_setting: {
    strong: ["freshwater", "river", "lake", "pond", "stream"],
    weak: ["spring"],
  },
  coastal_setting: {
    strong: ["coast", "coastal", "shore", "reef", "littoral", "tidepool"],
    weak: ["sea", "ocean"],
  },
  astral_setting: {
    strong: ["astral", "silver void"],
  },
  plane_of_fire_setting: {
    strong: ["plane of fire"],
  },
  plane_of_air_setting: {
    strong: ["plane of air"],
  },
  plane_of_water_setting: {
    strong: ["plane of water"],
  },
  plane_of_earth_setting: {
    strong: ["plane of earth"],
  },
  first_world_setting: {
    strong: ["first world", "fey realm"],
  },
  dreamlands_setting: {
    strong: ["dreamlands", "dreaming plane", "leng", "nightgaunt", "shantak"],
  },
  boneyard_setting: {
    strong: ["boneyard", "psychopomp"],
  },
  heaven_setting: {
    strong: ["heaven", "archon", "celestial mountain"],
  },
  nirvana_setting: {
    strong: ["nirvana", "agathion"],
  },
  elysium_setting: {
    strong: ["elysium", "azata"],
  },
  hell_setting: {
    strong: ["infernal", "diabolic"],
    weak: ["hell", "stygian"],
  },
  abyss_setting: {
    strong: ["abyss", "abyssal", "qlippoth"],
  },
  abaddon_setting: {
    strong: ["abaddon", "daemon"],
  },
  axis_setting: {
    strong: ["axis", "eternal city"],
  },
  shadow_plane_setting: {
    strong: ["shadow plane", "plane of shadow"],
    weak: ["umbral", "penumbral"],
  },
  maelstrom_setting: {
    strong: ["maelstrom", "protean"],
  },
  island_setting: {
    strong: ["island", "archipelago", "isle", "atoll"],
  },
  nautical_setting: {
    strong: ["nautical", "ship", "harbor", "dock", "wreck", "sailor", "pirate", "vessel"],
    weak: ["sea", "ocean"],
  },
  forest_setting: {
    strong: ["forest", "grove", "jungle", "rainforest", "woodland", "thicket", "briar"],
    weak: ["canopy", "arboreal"],
  },
  jungle_setting: {
    strong: ["jungle", "rainforest", "tropical forest"],
    weak: ["canopy"],
  },
  plains_setting: {
    strong: ["plains", "grassland", "prairie", "savanna", "savannah", "steppe"],
  },
  canyon_setting: {
    strong: ["canyon", "gorge", "mesa", "badlands"],
  },
  swamp_setting: {
    strong: ["swamp", "bog", "marsh", "fen", "mire", "bayou", "mangrove"],
  },
  underground_setting: {
    strong: ["underground", "subterranean", "cave", "cavern", "tunnel", "crypt", "sewer", "mine", "darklands", "orv"],
  },
  urban_setting: {
    strong: ["urban", "city", "street", "alley", "sewer", "metropolis", "culvert", "drain"],
    weak: ["market"],
  },
  battlefield_setting: {
    strong: ["battlefield", "battlefields", "war zone", "war zones", "front line", "front lines"],
  },
  geb_setting: {
    strong: ["geb", "gebbite", "gebbites", "graydirge"],
  },
  gravelands_setting: {
    strong: ["gravelands", "lastwall", "gallowspire", "fort ozem", "chernasardo"],
  },
  tian_xia_setting: {
    strong: ["tian xia", "tian-shu", "tian-hwan", "tian-sing", "tian-la", "minata", "bonmu", "goka", "minkai"],
  },
  small_settlement_setting: {
    strong: ["village", "villager", "hamlet", "small settlement", "outlying settlement"],
    weak: ["town", "townsfolk", "settlement"],
  },
  arctic_setting: {
    strong: ["arctic", "tundra", "glacier", "icebound"],
    weak: ["frozen", "snow"],
  },
  desert_setting: {
    strong: ["desert", "dune", "oasis"],
    weak: ["arid", "sand"],
  },
  wasteland_setting: {
    strong: ["wasteland", "wastes", "blasted"],
    weak: ["barren"],
  },
  mountain_setting: {
    strong: ["mountain", "peak", "cliff", "ridge", "foothills", "highlands", "crag", "mountain pass"],
  },
  sky_setting: {
    strong: ["open sky", "open skies", "high altitude", "high altitudes", "cloud top", "cloud tops", "aerial"],
    weak: ["sky", "skies", "cloud", "clouds"],
  },
  graveyard_setting: {
    strong: ["graveyard", "cemetery", "mausoleum", "barrow", "cairn", "burial ground", "tomb"],
  },
  ruins_setting: {
    strong: ["ruin", "ruins", "derelict", "collapsed", "crumbling", "lost city"],
  },
  temple_setting: {
    strong: ["temple", "shrine", "cathedral", "monastery", "chapel", "sanctuary", "abbey", "holy site", "cloister", "ziggurat"],
  },
  fortress_setting: {
    strong: ["fortress", "citadel", "keep", "stronghold", "bastion", "watchtower", "garrison", "castle", "fort", "battlement", "rampart"],
  },
  volcanic_setting: {
    strong: ["volcanic", "volcano", "lava", "magma", "caldera", "cinder"],
    weak: ["ash", "ashen"],
  },
  rural_setting: {
    strong: ["rural", "farm", "pasture", "countryside", "cropland", "mill", "homestead"],
  },
};

const NOVEL_SETTING_STRONG_CUES = [
  "erebus",
  "stygia",
];

const NOVEL_SETTING_WEAK_CUES = [
  "penumbral",
  "umbral",
];

const AMBIGUOUS_SINGLE_TOKEN_CUES = new Set([
  "ash",
  "ashen",
  "cloud",
  "clouds",
  "hell",
  "market",
  "ocean",
  "sand",
  "sea",
  "sky",
  "skies",
  "snow",
  "spring",
  "town",
  "umbral",
]);

function normalizeSurface(value: string): string {
  return value
    .replace(/^target:/, "")
    .replace(/^scope:/, "")
    .replace(/_/g, " ")
    .trim()
    .toLowerCase()
    .replace(/\s+/g, " ");
}

function countSurfaceTokens(surface: string): number {
  return surface.split(" ").filter(Boolean).length;
}

function escapeRegex(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function surfaceIncludesCue(surface: string, candidate: string): boolean {
  const normalizedCandidate = normalizeSurface(candidate);
  if (!normalizedCandidate) {
    return false;
  }

  const pattern = new RegExp(`(^|[^a-z0-9])${escapeRegex(normalizedCandidate)}([^a-z0-9]|$)`);
  return pattern.test(surface);
}

function collectCueMatches(
  surface: string,
  cueSpecs: Record<string, SettingCueSpec>,
): SettingCueMatch[] {
  const matches: SettingCueMatch[] = [];
  for (const [tag, spec] of Object.entries(cueSpecs)) {
    for (const cue of spec.strong) {
      if (surfaceIncludesCue(surface, cue)) {
        matches.push({
          tag: normalizeDerivedTag(tag),
          cue: normalizeSurface(cue),
          strength: "strong",
        });
      }
    }
    for (const cue of spec.weak ?? []) {
      if (surfaceIncludesCue(surface, cue)) {
        matches.push({
          tag: normalizeDerivedTag(tag),
          cue: normalizeSurface(cue),
          strength: "weak",
        });
      }
    }
  }

  for (const cue of NOVEL_SETTING_STRONG_CUES) {
    if (surfaceIncludesCue(surface, cue)) {
      matches.push({
        tag: null,
        cue: normalizeSurface(cue),
        strength: "novel",
      });
    }
  }
  for (const cue of NOVEL_SETTING_WEAK_CUES) {
    if (surfaceIncludesCue(surface, cue)) {
      matches.push({
        tag: null,
        cue: normalizeSurface(cue),
        strength: "weak",
      });
    }
  }

  return matches;
}

function choosePrimaryCue(matches: SettingCueMatch[]): SettingCueMatch | null {
  return matches
    .slice()
    .sort((left, right) =>
      strengthPriority(right.strength) - strengthPriority(left.strength) ||
      right.cue.length - left.cue.length ||
      left.cue.localeCompare(right.cue))[0] ?? null;
}

function strengthPriority(value: SettingCueMatch["strength"]): number {
  if (value === "novel") {
    return 3;
  }
  if (value === "strong") {
    return 2;
  }
  if (value === "weak") {
    return 1;
  }
  return 0;
}

function computeCueLocality(
  surface: string,
  matches: SettingCueMatch[],
): number {
  if (matches.length === 0) {
    return 0.35;
  }

  const primary = choosePrimaryCue(matches);
  const tokenCount = countSurfaceTokens(surface);
  const overlapCount = new Set(matches.map((match) => match.tag).filter((tag): tag is string => tag !== null)).size;
  const base = primary?.strength === "novel"
    ? 1
    : primary?.strength === "strong"
      ? tokenCount > 1 ? 1 : 0.96
      : tokenCount > 1 ? 0.7 : 0.48;
  const overlapPenalty = overlapCount > 2 ? 0.18 : overlapCount === 2 ? 0.08 : 0;
  return Math.max(0.3, Math.min(1, base - overlapPenalty));
}

function computeCueAmbiguityPenalty(matches: SettingCueMatch[]): number {
  const strongMatches = matches.filter((match) => match.strength === "strong" || match.strength === "novel");
  const overlapCount = new Set(strongMatches.map((match) => match.tag).filter((tag): tag is string => tag !== null)).size;
  if (overlapCount >= 3) {
    return 0.28;
  }
  if (overlapCount === 2) {
    return 0.12;
  }
  if (matches.some((match) => match.strength === "weak")) {
    return 0.08;
  }
  return 0;
}

function computeBoilerplateRisk(surface: string, matches: SettingCueMatch[]): number {
  if (matches.length === 0) {
    return 0.4;
  }

  const tokenCount = countSurfaceTokens(surface);
  const primaryCue = choosePrimaryCue(matches)?.cue ?? "";
  const weakOnly = matches.every((match) => match.strength === "weak");
  const ambiguousSingleToken = tokenCount === 1 && AMBIGUOUS_SINGLE_TOKEN_CUES.has(primaryCue);
  if (weakOnly) {
    return ambiguousSingleToken ? 0.72 : 0.48;
  }
  if (ambiguousSingleToken) {
    return 0.35;
  }
  return tokenCount > 2 ? 0.04 : 0.1;
}

function computeFamilyConceptWeight(
  matches: SettingCueMatch[],
  cueLocality: number,
  cueAmbiguityPenalty: number,
  boilerplateRisk: number,
): number {
  const primary = choosePrimaryCue(matches);
  const baseWeight = primary?.strength === "novel"
    ? 1.8
    : primary?.strength === "strong"
      ? 1.35
      : primary?.strength === "weak"
        ? 0.55
        : 0.45;
  return Math.max(
    0.18,
    baseWeight +
    (cueLocality * 0.18) -
    cueAmbiguityPenalty -
    (boilerplateRisk * 0.45),
  );
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
      cueStrength: "none",
      cueLocality: 1,
      cueAmbiguityPenalty: 0,
      boilerplateRisk: 0,
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
      cueStrength: "none",
      cueLocality: 0.3,
      cueAmbiguityPenalty: 0,
      boilerplateRisk: 0.3,
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
      cueStrength: "none",
      cueLocality: 0.2,
      cueAmbiguityPenalty: 0.1,
      boilerplateRisk: 0.85,
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
      cueStrength: "none",
      cueLocality: 0.25,
      cueAmbiguityPenalty: 0.1,
      boilerplateRisk: 0.45,
    };
  }

  const cueMatches = collectCueMatches(surface, EXISTING_SETTING_TAG_CUES);
  const existingTagOverlaps = uniqueTags(cueMatches);
  const primaryMatch = choosePrimaryCue(cueMatches);
  const primaryCue = primaryMatch?.cue ?? null;
  const cueStrength = primaryMatch?.strength ?? "none";
  const cueLocality = computeCueLocality(surface, cueMatches);
  const cueAmbiguityPenalty = computeCueAmbiguityPenalty(cueMatches);
  const boilerplateRisk = computeBoilerplateRisk(surface, cueMatches);
  const hasStrongAnchor = cueMatches.some((match) => match.strength === "strong" || match.strength === "novel");
  const weakOnlyCue = cueMatches.length > 0 && !hasStrongAnchor;
  const isAmbiguousWeakCue = weakOnlyCue && countSurfaceTokens(surface) === 1;

  if (hasStrongAnchor) {
    return {
      bucket: "possible_place_anchor",
      eligibleForClustering: true,
      familyConceptWeight: computeFamilyConceptWeight(cueMatches, cueLocality, cueAmbiguityPenalty, boilerplateRisk),
      existingTagOverlaps,
      primaryCue,
      suppressionReason: null,
      cueStrength,
      cueLocality,
      cueAmbiguityPenalty,
      boilerplateRisk,
    };
  }

  if (weakOnlyCue || isAmbiguousWeakCue) {
    return {
      bucket: "context_only",
      eligibleForClustering: false,
      familyConceptWeight: computeFamilyConceptWeight(cueMatches, cueLocality, cueAmbiguityPenalty, boilerplateRisk),
      existingTagOverlaps,
      primaryCue,
      suppressionReason: "weak-place-cue",
      cueStrength,
      cueLocality,
      cueAmbiguityPenalty,
      boilerplateRisk,
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
      cueStrength: "none",
      cueLocality: 0.35,
      cueAmbiguityPenalty: 0,
      boilerplateRisk: 0.35,
    };
  }

  return {
    bucket: "unknown",
    eligibleForClustering: false,
    familyConceptWeight: 0.55,
    existingTagOverlaps: [],
    primaryCue: null,
    suppressionReason: "not-setting-salient",
    cueStrength: "none",
    cueLocality: 0.35,
    cueAmbiguityPenalty: 0,
    boilerplateRisk: 0.4,
  };
}

function uniqueTags(matches: SettingCueMatch[]): string[] {
  return [...new Set(matches.map((match) => match.tag).filter((tag): tag is string => tag !== null))].sort();
}

function includesAny(surface: string, candidates: Iterable<string>): boolean {
  for (const candidate of candidates) {
    if (!candidate) {
      continue;
    }
    if (surfaceIncludesCue(surface, candidate)) {
      return true;
    }
  }
  return false;
}
