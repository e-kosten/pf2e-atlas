import { mkdtemp, mkdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { DatabaseSync } from "node:sqlite";

import { afterEach, describe, expect, it } from "vitest";

import {
  createPf2eApplicationStorageService,
  openConfiguredPf2eApplicationIndex,
  type Pf2eApplicationIndexHandle,
} from "../../src/app/storage-service.js";
import type { AppConfig } from "../../src/domain/config-types.js";

function createTestConfig(rootPath: string, indexPath: string): AppConfig {
  return {
    dataPath: rootPath,
    rootPath,
    manifestPath: path.join(rootPath, "system.pf2e.json"),
    indexPath,
    embeddings: {
      provider: "hash",
      modelId: "test-model",
      modelRevision: null,
      cachePath: path.join(rootPath, ".cache", "models"),
      localModelPath: null,
    },
    ranking: {
      configPath: path.join(rootPath, "pf2e-ranking.json"),
      watch: false,
    },
  };
}

async function createConfiguredIndexFixture(): Promise<{ config: AppConfig; rootPath: string }> {
  const rootPath = await mkdtemp(path.join(tmpdir(), "pf2e-storage-service-"));
  const indexPath = path.join(rootPath, "pf2e-index.sqlite");
  await mkdir(path.join(rootPath, ".cache"), { recursive: true });
  await writeFile(path.join(rootPath, "system.pf2e.json"), "{}\n", "utf8");
  const db = new DatabaseSync(indexPath);
  db.exec("CREATE TABLE sanity (value TEXT NOT NULL); INSERT INTO sanity (value) VALUES ('ready');");
  db.close();
  return {
    config: createTestConfig(rootPath, indexPath),
    rootPath,
  };
}

describe("application storage service", () => {
  const tempRoots: string[] = [];
  const openHandles: Pf2eApplicationIndexHandle[] = [];

  afterEach(async () => {
    while (openHandles.length > 0) {
      const handle = openHandles.pop();
      handle?.close();
    }
    await Promise.all(tempRoots.splice(0).map((rootPath) => rm(rootPath, { force: true, recursive: true })));
  });

  it("opens a configured application index through the shared storage owner", async () => {
    const fixture = await createConfiguredIndexFixture();
    tempRoots.push(fixture.rootPath);

    const handle = await openConfiguredPf2eApplicationIndex([
      "--data-path",
      fixture.rootPath,
      "--index-path",
      fixture.config.indexPath,
    ]);
    openHandles.push(handle);

    expect(handle.config.indexPath).toBe(fixture.config.indexPath);
    expect(handle.db.prepare("SELECT value FROM sanity").get()).toEqual({ value: "ready" });
  });

  it("validates the configured index path before opening it", async () => {
    const fixture = await createConfiguredIndexFixture();
    tempRoots.push(fixture.rootPath);

    const storage = createPf2eApplicationStorageService({
      ...fixture.config,
      indexPath: path.join(fixture.rootPath, "missing.sqlite"),
    });

    await expect(storage.openIndex()).rejects.toThrow(/missing\.sqlite/);
  });
});
