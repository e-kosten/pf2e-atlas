import { mkdtemp, mkdir } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { writeJson } from "./pf2e-fixture.js";
import { writeCreatureFixtureData } from "./pf2e-service-fixture-main-creatures.js";
import { writeRulesAndItemsFixtureData } from "./pf2e-service-fixture-main-rules-items.js";
import { writeSpellAndAfflictionFixtureData } from "./pf2e-service-fixture-main-spells-afflictions.js";
import {
  cleanupCreatedRoots,
  ServiceTestFixture,
} from "./pf2e-service-fixture-runtime.js";

export async function createFixture(): Promise<ServiceTestFixture> {
  const root = await mkdtemp(path.join(os.tmpdir(), "pf2e-mcp-test-"));
  const packRoot = path.join(root, "packs", "pf2e");
  const packNames = [
    "actions",
    "actionspf2e",
    "afflictions",
    "bestiary-family-ability-glossary",
    "classfeatures",
    "conditionitems",
    "equipment",
    "equipment-srd",
    "feats-srd",
    "heritages",
    "journals",
    "pathfinder-monster-core",
    "pathfinder-npc-core",
    "pfs-season-1-bestiary",
    "quest-for-the-frozen-flame-bestiary",
    "spells",
    "spells-srd",
  ];

  await Promise.all(packNames.map(async (packName) => mkdir(path.join(packRoot, packName), { recursive: true })));
  await Promise.all([
    mkdir(path.join(packRoot, "bestiary-family-ability-glossary", "ghost"), { recursive: true }),
    mkdir(path.join(packRoot, "bestiary-family-ability-glossary", "lich"), { recursive: true }),
    mkdir(path.join(packRoot, "bestiary-family-ability-glossary", "mythic"), { recursive: true }),
    mkdir(path.join(packRoot, "bestiary-family-ability-glossary", "vampire"), { recursive: true }),
    mkdir(path.join(packRoot, "pathfinder-npc-core", "seafarer"), { recursive: true }),
  ]);
  await mkdir(path.join(root, "src", "module", "migration", "migrations"), { recursive: true });

  await writeJson(path.join(root, "system.pf2e.json"), {
    packs: [
      { name: "actions", label: "Actions", path: "packs/actions", type: "Item" },
      { name: "actionspf2e", label: "Actions SRD", path: "packs/actionspf2e", type: "Item" },
      { name: "afflictions", label: "Afflictions", path: "packs/afflictions", type: "Item" },
      {
        name: "bestiary-family-ability-glossary",
        label: "Creature Family Ability Glossary",
        path: "packs/bestiary-family-ability-glossary",
        type: "Item",
      },
      { name: "classfeatures", label: "Class Features", path: "packs/classfeatures", type: "Item" },
      { name: "conditionitems", label: "Conditions", path: "packs/conditionitems", type: "Item" },
      { name: "equipment", label: "Equipment", path: "packs/equipment", type: "Item" },
      { name: "equipment-srd", label: "Equipment SRD", path: "packs/equipment-srd", type: "Item" },
      { name: "feats-srd", label: "Feats", path: "packs/feats-srd", type: "Item" },
      { name: "heritages", label: "Heritages", path: "packs/heritages", type: "Item" },
      { name: "journals", label: "Journals", path: "packs/journals", type: "JournalEntry" },
      { name: "pathfinder-npc-core", label: "Pathfinder NPC Core", path: "packs/pathfinder-npc-core", type: "Actor" },
      { name: "pathfinder-monster-core", label: "Pathfinder Monster Core", path: "packs/pathfinder-monster-core", type: "Actor" },
      { name: "pfs-season-1-bestiary", label: "Season 1", path: "packs/pfs-season-1-bestiary", type: "Actor" },
      {
        name: "quest-for-the-frozen-flame-bestiary",
        label: "Quest for the Frozen Flame",
        path: "packs/quest-for-the-frozen-flame-bestiary",
        type: "Actor",
      },
      { name: "spells", label: "Spells", path: "packs/spells", type: "Item" },
      { name: "spells-srd", label: "Spells SRD", path: "packs/spells-srd", type: "Item" },
    ],
  });

  await writeRulesAndItemsFixtureData(root, packRoot);
  await writeSpellAndAfflictionFixtureData(packRoot);
  await writeCreatureFixtureData(packRoot);

  return {
    root,
    manifestPath: path.join(root, "system.pf2e.json"),
  };
}

export { createHardFilterFixture } from "./pf2e-service-fixture-hard-filters.js";
export { cleanupCreatedRoots } from "./pf2e-service-fixture-runtime.js";
export type { ServiceTestFixture } from "./pf2e-service-fixture-runtime.js";
