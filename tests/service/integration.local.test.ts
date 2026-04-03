import { access, mkdtemp, rm, stat } from "node:fs/promises";
import { constants } from "node:fs";
import os from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { Pf2eDataService } from "../../src/data/service.js";

const TEST_HASH_EMBEDDING = {
  provider: "hash" as const,
  modelId: "feature-hash-192",
  modelRevision: null,
  cachePath: path.join(os.tmpdir(), "pf2e-local-integration-hf-cache"),
  localModelPath: null,
};

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
  const createdRoots: string[] = [];

  afterEach(async () => {
    await Promise.all(
      createdRoots.splice(0).map(async (root) => rm(root, { recursive: true, force: true })),
    );
  });

  it.runIf(available)("rebuilds a fresh SQLite index and can resolve known records", async () => {
    const tempRoot = await mkdtemp(path.join(os.tmpdir(), "pf2e-local-integration-"));
    createdRoots.push(tempRoot);
    const indexPath = path.join(tempRoot, "pf2e-index.sqlite");
    const service = await Pf2eDataService.rebuildIndex(localRoot, manifestPath, {
      indexPath,
      embedding: TEST_HASH_EMBEDDING,
    });

    expect(await access(indexPath, constants.R_OK).then(() => true)).toBe(true);
    expect(service.listPacks().length).toBeGreaterThan(50);
    expect(service.lookup("Raise a Shield").match?.packLabel).toBe("Actions");
    expect(service.lookup("Analysis Eye").match?.packLabel).toBe("Equipment");
    expect(service.lookup("Cythnigot", { category: "creature" }).match?.type).toBe("npc");
    service.close();
  }, 60000);

  it.runIf(available)("reuses a cached SQLite index when the PF2E source is unchanged", async () => {
    const tempRoot = await mkdtemp(path.join(os.tmpdir(), "pf2e-local-cache-"));
    createdRoots.push(tempRoot);
    const indexPath = path.join(tempRoot, "pf2e-index.sqlite");

    const firstService = await Pf2eDataService.rebuildIndex(localRoot, manifestPath, {
      indexPath,
      embedding: TEST_HASH_EMBEDDING,
    });
    expect(firstService.lookup("Raise a Shield").match?.packLabel).toBe("Actions");
    firstService.close();

    const firstMtime = (await stat(indexPath)).mtimeMs;
    const secondService = await Pf2eDataService.load(localRoot, manifestPath, {
      indexPath,
      embedding: TEST_HASH_EMBEDDING,
    });
    expect(secondService.lookup("Analysis Eye").match?.packLabel).toBe("Equipment");
    secondService.close();
    const secondMtime = (await stat(indexPath)).mtimeMs;

    expect(secondMtime).toBe(firstMtime);
  }, 60000);
});
