import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { loadConfig } from "../src/config.js";

describe("loadConfig", () => {
  const createdRoots: string[] = [];

  afterEach(async () => {
    await Promise.all(
      createdRoots.splice(0).map(async (root) => {
        await import("node:fs/promises").then(({ rm }) => rm(root, { recursive: true, force: true }));
      }),
    );
  });

  async function createRepoFixture(): Promise<string> {
    const root = await mkdtemp(path.join(os.tmpdir(), "pf2e-config-test-"));
    const dataRoot = path.join(root, "vendor", "pf2e");
    await mkdir(dataRoot, { recursive: true });
    await writeFile(path.join(dataRoot, "system.pf2e.json"), JSON.stringify({ packs: [] }));
    createdRoots.push(root);
    return root;
  }

  it("prefers the CLI data path", async () => {
    const root = await createRepoFixture();
    const explicitDataPath = path.join(root, "custom", "pf2e");
    const explicitIndexPath = path.join(root, "custom", "pf2e-index.sqlite");
    await mkdir(explicitDataPath, { recursive: true });
    await writeFile(path.join(explicitDataPath, "system.pf2e.json"), JSON.stringify({ packs: [] }));

    const config = await loadConfig(
      ["--data-path", explicitDataPath, "--index-path", explicitIndexPath],
      { PF2E_DATA_PATH: "/tmp/ignored", PF2E_INDEX_PATH: "/tmp/ignored.sqlite" },
    );

    expect(config.rootPath).toBe(explicitDataPath);
    expect(config.manifestPath).toBe(path.join(explicitDataPath, "system.pf2e.json"));
    expect(config.indexPath).toBe(explicitIndexPath);
  });

  it("defaults to vendor/pf2e under the current working directory", async () => {
    const root = await createRepoFixture();
    const originalCwd = process.cwd();

    try {
      process.chdir(root);
      const config = await loadConfig([], {});
      expect(config.rootPath.endsWith(path.join("vendor", "pf2e"))).toBe(true);
      expect(config.manifestPath.endsWith(path.join("vendor", "pf2e", "system.pf2e.json"))).toBe(true);
      expect(config.indexPath.endsWith(path.join(".cache", "pf2e-index.sqlite"))).toBe(true);
    } finally {
      process.chdir(originalCwd);
    }
  });

  it("throws when the default path does not contain a manifest", async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "pf2e-config-missing-"));
    createdRoots.push(root);
    const originalCwd = process.cwd();

    try {
      process.chdir(root);
      await expect(loadConfig([], {})).rejects.toThrow(/Clone the PF2E repo into vendor\/pf2e/);
    } finally {
      process.chdir(originalCwd);
    }
  });
});
