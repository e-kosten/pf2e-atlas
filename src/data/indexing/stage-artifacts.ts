import type {
  ActorIndexData,
  BuildSourceEntry,
  IndexedBuildSourceEntry,
  ItemIndexData,
  NormalizedIndexRecord,
  SpellIndexData,
} from "../index-types.js";

function cloneActorIndexData(value: ActorIndexData | null): ActorIndexData | null {
  if (!value) {
    return null;
  }

  return {
    ...value,
    languages: [...value.languages],
    speedTypes: [...value.speedTypes],
    senses: [...value.senses],
    immunities: [...value.immunities],
    resistances: [...value.resistances],
    weaknesses: [...value.weaknesses],
    disableSkills: [...value.disableSkills],
    actorMetrics: { ...value.actorMetrics },
  };
}

function cloneItemIndexData(value: ItemIndexData | null): ItemIndexData | null {
  if (!value) {
    return null;
  }

  return {
    ...value,
    damageTypes: [...value.damageTypes],
    itemMetrics: { ...value.itemMetrics },
  };
}

function cloneSpellIndexData(value: SpellIndexData | null): SpellIndexData | null {
  if (!value) {
    return null;
  }

  return {
    ...value,
    traditions: [...value.traditions],
    spellKinds: [...value.spellKinds],
    damageTypes: [...value.damageTypes],
  };
}

export function cloneNormalizedIndexRecord(record: NormalizedIndexRecord): NormalizedIndexRecord {
  return {
    ...record,
    traits: [...record.traits],
    derivedTags: [...record.derivedTags],
    families: [...record.families],
    variantAxes: [...record.variantAxes],
    damageTypes: [...record.damageTypes],
    traditions: [...record.traditions],
    spellKinds: [...record.spellKinds],
    languages: [...record.languages],
    speedTypes: [...record.speedTypes],
    senses: [...record.senses],
    immunities: [...record.immunities],
    resistances: [...record.resistances],
    weaknesses: [...record.weaknesses],
    disableSkills: [...record.disableSkills],
    actorMetrics: { ...record.actorMetrics },
    itemMetrics: { ...record.itemMetrics },
  };
}

export function cloneIndexedBuildSourceEntry(entry: IndexedBuildSourceEntry): IndexedBuildSourceEntry {
  return {
    ...entry,
    record: cloneNormalizedIndexRecord(entry.record),
    actorData: cloneActorIndexData(entry.actorData),
    itemData: cloneItemIndexData(entry.itemData),
    spellData: cloneSpellIndexData(entry.spellData),
    references: [...entry.references],
    resolvedReferences: [...entry.resolvedReferences],
  };
}

export function toIndexedBuildSourceEntry(entry: BuildSourceEntry): IndexedBuildSourceEntry | null {
  if (entry.record === null) {
    return null;
  }

  return cloneIndexedBuildSourceEntry({
    ...entry,
    record: entry.record,
  });
}
