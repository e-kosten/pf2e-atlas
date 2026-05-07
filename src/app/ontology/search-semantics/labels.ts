import { formatOntologySearchVocabularyLabel } from "../../../domain/presentation-vocabulary.js";
import type { DerivedTagCatalogEntry, DerivedTagCatalogTag } from "../../../domain/record-types.js";
import type { SearchCategory, SearchSubcategory } from "../../../domain/search-types.js";
import { titleCaseLabel } from "../node-helpers.js";

const METRIC_SEGMENT_LABELS: Readonly<Record<string, string>> = {
  ac: "AC",
  ac_bonus: "AC Bonus",
  arcana: "Arcana",
  athletics: "Athletics",
  best: "Best",
  bt: "Broken Threshold",
  cha: "Charisma",
  check_penalty: "Check Penalty",
  con: "Constitution",
  crafting: "Crafting",
  damage_dice: "Damage Dice",
  damage_die_faces: "Damage Die Faces",
  dc: "DC",
  dex: "Dexterity",
  dex_cap: "Dex Cap",
  faces: "Faces",
  fly: "Fly",
  fort: "Fortitude",
  hardness: "Hardness",
  hp: "HP",
  int: "Intelligence",
  land: "Land",
  max: "Maximum",
  min: "Minimum",
  mod: "Modifier",
  perception: "Perception",
  proficient: "Proficient",
  range: "Range",
  range_increment: "Range Increment",
  rank: "Rank",
  ref: "Reflex",
  religion: "Religion",
  reload: "Reload",
  scent: "Scent",
  speed_penalty: "Speed Penalty",
  str: "Strength",
  strength: "Strength",
  thievery: "Thievery",
  value: "Value",
  will: "Will",
  wis: "Wisdom",
  worst: "Worst",
};

export type DerivedTagLabels = {
  categoryLabel: string;
  activeSubcategoryLabel: string;
  familyLabel: string;
  axisLabel: string;
  tagLabel: string;
  familyScopeLabel: string;
  assignmentModeLabel: string;
};

function humanizeMetricSegment(segment: string): string {
  return METRIC_SEGMENT_LABELS[segment] ?? titleCaseLabel(segment);
}

export function formatSearchSemanticsMetricLabel(metricKey: string): string {
  const segments = metricKey.split(".");
  const [first, second, third, fourth] = segments;

  if (first === "ability" && second && third === "mod") {
    return `${humanizeMetricSegment(second)} Modifier`;
  }
  if (first === "perception" && second === "mod") {
    return "Perception Modifier";
  }
  if (first === "ac" && second === "value") {
    return "Armor Class";
  }
  if (first === "hardness" && second === "value") {
    return "Hardness";
  }
  if (first === "hp" && second === "value") {
    return "Hit Points";
  }
  if (first === "hp" && second === "max") {
    return "Maximum Hit Points";
  }
  if (first === "hp" && second === "bt") {
    return "Broken Threshold";
  }
  if (first === "save" && second && third === "mod") {
    return `${humanizeMetricSegment(second)} Save Modifier`;
  }
  if (first === "save" && second === "best") {
    return "Best Save";
  }
  if (first === "save" && second === "worst") {
    return "Worst Save";
  }
  if (first === "skill" && second && third === "mod") {
    return `${humanizeMetricSegment(second)} Modifier`;
  }
  if (first === "skill" && second && third === "rank") {
    return `${humanizeMetricSegment(second)} Rank`;
  }
  if (first === "skill" && second && third === "proficient") {
    return `${humanizeMetricSegment(second)} Proficient`;
  }
  if (first === "stealth" && second === "mod") {
    return "Stealth Modifier";
  }
  if (first === "stealth" && second === "dc") {
    return "Stealth DC";
  }
  if (first === "speed" && second && third === "value") {
    return `${humanizeMetricSegment(second)} Speed`;
  }
  if (first === "sense" && second && third === "range") {
    return `${humanizeMetricSegment(second)} Range`;
  }
  if (first === "disable" && second === "dc" && third === "min") {
    return "Minimum Disable DC";
  }
  if (first === "disable" && second === "dc" && third === "max") {
    return "Maximum Disable DC";
  }
  if (first === "disable" && second && third === "dc" && fourth === "min") {
    return `Minimum ${humanizeMetricSegment(second)} Disable DC`;
  }
  if (first === "disable" && second && third === "dc" && fourth === "max") {
    return `Maximum ${humanizeMetricSegment(second)} Disable DC`;
  }
  if (first === "disable" && second && third === "rank" && fourth === "min") {
    return `Minimum ${humanizeMetricSegment(second)} Disable Rank`;
  }
  if (first === "weapon" && second) {
    return `Weapon ${humanizeMetricSegment(second)}`;
  }
  if (first === "armor" && second) {
    return `Armor ${humanizeMetricSegment(second)}`;
  }
  if (first === "shield" && second) {
    return `Shield ${humanizeMetricSegment(second)}`;
  }

  return segments.map((segment) => humanizeMetricSegment(segment)).join(" ");
}

export function formatDerivedTagFamilyScopeLabel(familyEntry: DerivedTagCatalogEntry): string {
  return (
    familyEntry.subcategories?.map((entry) => formatOntologySearchVocabularyLabel(entry)).join(", ") ??
    "(all subcategories)"
  );
}

export function getDerivedTagLabels(
  category: SearchCategory,
  activeSubcategory: SearchSubcategory | null,
  familyEntry: DerivedTagCatalogEntry,
  tag: DerivedTagCatalogTag,
): DerivedTagLabels {
  return {
    categoryLabel: formatOntologySearchVocabularyLabel(category),
    activeSubcategoryLabel: activeSubcategory ? formatOntologySearchVocabularyLabel(activeSubcategory) : "(all)",
    familyLabel: formatOntologySearchVocabularyLabel(familyEntry.family),
    axisLabel: formatOntologySearchVocabularyLabel(familyEntry.axis),
    tagLabel: formatOntologySearchVocabularyLabel(tag.value),
    familyScopeLabel: formatDerivedTagFamilyScopeLabel(familyEntry),
    assignmentModeLabel: tag.assignmentMode
      ? formatOntologySearchVocabularyLabel(tag.assignmentMode)
      : familyEntry.assignmentMode
        ? formatOntologySearchVocabularyLabel(familyEntry.assignmentMode)
        : "(unspecified)",
  };
}
