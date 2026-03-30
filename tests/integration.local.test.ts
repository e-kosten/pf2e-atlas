import { access } from "node:fs/promises";
import { constants } from "node:fs";
import path from "node:path";

import { describe, expect, it } from "vitest";

import { Pf2eDataService } from "../src/pf2e-data.js";

const localRoot = process.env.PF2E_DATA_PATH ?? path.resolve(process.cwd(), "vendor", "pf2e");
const manifestPath = `${localRoot}/system.pf2e.json`;

async function hasLocalData(): Promise<boolean> {
  try {
    await access(manifestPath, constants.R_OK);
    return true;
  } catch {
    return false;
  }
}

describe("local PF2E integration", async () => {
  const available = await hasLocalData();

  it.runIf(available)("loads the local PF2E export and can resolve known records", async () => {
    const service = await Pf2eDataService.load(localRoot, manifestPath);

    expect(service.listPacks().length).toBeGreaterThan(50);
    expect(service.lookup("Raise a Shield").match?.packLabel).toBe("Actions");
    expect(service.lookup("Analysis Eye").match?.packLabel).toBe("Equipment");
    expect(service.lookup("Cythnigot", { documentType: "Actor" }).match?.type).toBe("npc");
  });
});
