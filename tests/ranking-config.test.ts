import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";

import { afterEach, describe, expect, it } from "vitest";

import { DEFAULT_RANKING_CONFIG, mergeRankingConfig, RankingConfigStore } from "../src/ranking-config.js";

describe("ranking config", () => {
  const createdRoots: string[] = [];

  afterEach(async () => {
    await Promise.all(
      createdRoots.splice(0).map(async (root) => {
        await import("node:fs/promises").then(({ rm }) => rm(root, { recursive: true, force: true }));
      }),
    );
  });

  it("merges partial overrides into the default ranking config", () => {
    const merged = mergeRankingConfig({
      lexicalChannels: {
        themeTraits: 0.5,
      },
      hybridFusion: {
        balanced: {
          lexicalWeight: 3,
          semanticWeight: 1,
          lexicalTopK: 90,
        },
      },
      sourceQuality: {
        adventure: 0.2,
      },
    });

    expect(merged.lexicalChannels.themeTraits).toBe(0.5);
    expect(merged.lexicalChannels.themeName).toBe(DEFAULT_RANKING_CONFIG.lexicalChannels.themeName);
    expect(merged.hybridFusion.balanced.lexicalWeight).toBeCloseTo(0.75);
    expect(merged.hybridFusion.balanced.semanticWeight).toBeCloseTo(0.25);
    expect(merged.hybridFusion.balanced.lexicalTopK).toBe(90);
    expect(merged.hybridFusion.concept.semanticTopK).toBe(DEFAULT_RANKING_CONFIG.hybridFusion.concept.semanticTopK);
    expect(merged.sourceQuality.adventure).toBe(0.2);
  });

  it("falls back to defaults and records an error when the config file is invalid", async () => {
    const root = await mkdtemp(path.join(os.tmpdir(), "pf2e-ranking-config-test-"));
    createdRoots.push(root);
    await mkdir(path.join(root, "config"), { recursive: true });
    const configPath = path.join(root, "config", "ranking.json");
    await writeFile(configPath, "{not valid json");

    const store = await RankingConfigStore.create(configPath, { watch: false });

    expect(store.getConfig()).toEqual(DEFAULT_RANKING_CONFIG);
    expect(store.getStatus().source).toBe("default");
    expect(store.getStatus().lastError).toMatch(/Failed to load ranking config/);
    store.close();
  });
});
