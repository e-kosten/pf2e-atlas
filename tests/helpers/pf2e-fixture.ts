import { execFile } from "node:child_process";
import { writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";

import { Pf2eDataService } from "../../src/data/service.js";

const execFileAsync = promisify(execFile);

export const TEST_HASH_EMBEDDING = {
  provider: "hash" as const,
  modelId: "feature-hash-192",
  modelRevision: null,
  cachePath: path.join(os.tmpdir(), "pf2e-test-hf-cache"),
  localModelPath: null,
};

export async function writeJson(filePath: string, value: unknown): Promise<void> {
  await writeFile(filePath, JSON.stringify(value, null, 2));
}

export async function loadTestService(
  fixture: { root: string; manifestPath: string },
  options: Parameters<typeof Pf2eDataService.load>[2] = {},
): Promise<Pf2eDataService> {
  return Pf2eDataService.rebuildIndex(fixture.root, fixture.manifestPath, {
    embedding: TEST_HASH_EMBEDDING,
    ...options,
  });
}

export async function openPreparedTestService(
  fixture: { root: string; manifestPath: string },
  options: Parameters<typeof Pf2eDataService.load>[2] = {},
): Promise<Pf2eDataService> {
  return Pf2eDataService.load(fixture.root, fixture.manifestPath, {
    embedding: TEST_HASH_EMBEDDING,
    ...options,
  });
}

export async function initializeGitFixture(root: string): Promise<void> {
  await execFileAsync("git", ["init", "-b", "main"], { cwd: root });
  await execFileAsync("git", ["config", "user.name", "PF2E Test"], { cwd: root });
  await execFileAsync("git", ["config", "user.email", "pf2e-test@example.com"], { cwd: root });
  await execFileAsync("git", ["add", "."], { cwd: root });
  await execFileAsync("git", ["commit", "-m", "Initial fixture"], { cwd: root });
}

export function createFakeEmbeddingProviderFactory(
  identity: { provider: "hash" | "hf-local"; model: string; revision: string | null; dimensions: number },
  warnings: string[] = [],
): NonNullable<Parameters<typeof Pf2eDataService.load>[2]>["embeddingProviderFactory"] {
  const buildVector = (text: string): Float32Array => {
    const vector = new Float32Array(identity.dimensions);
    if (text.trim().length > 0) {
      vector[0] = 1;
    }
    return vector;
  };

  return async () => ({
    provider: {
      identity,
      async embed(text: string): Promise<Float32Array> {
        return buildVector(text);
      },
      async embedMany(texts: string[]): Promise<Float32Array[]> {
        return texts.map((text) => buildVector(text));
      },
    },
    warnings,
  });
}

export function createCapturingEmbeddingProviderFactory(
  calls: string[],
  identity: { provider: "hash" | "hf-local"; model: string; revision: string | null; dimensions: number },
): NonNullable<Parameters<typeof Pf2eDataService.load>[2]>["embeddingProviderFactory"] {
  const buildVector = (text: string): Float32Array => {
    calls.push(text);
    const vector = new Float32Array(identity.dimensions);
    if (text.trim().length > 0) {
      vector[0] = 1;
    }
    return vector;
  };

  return async () => ({
    provider: {
      identity,
      async embed(text: string): Promise<Float32Array> {
        return buildVector(text);
      },
      async embedMany(texts: string[]): Promise<Float32Array[]> {
        return texts.map((text) => buildVector(text));
      },
    },
    warnings: [],
  });
}

export function createEmbeddingBatchTrackingProviderFactory(
  state: { embedCalls: string[]; embedManyCalls: string[][] },
  identity: { provider: "hash" | "hf-local"; model: string; revision: string | null; dimensions: number },
): NonNullable<Parameters<typeof Pf2eDataService.load>[2]>["embeddingProviderFactory"] {
  const buildVector = (text: string): Float32Array => {
    const vector = new Float32Array(identity.dimensions);
    if (text.trim().length > 0) {
      vector[0] = 1;
    }
    return vector;
  };

  return async () => ({
    provider: {
      identity,
      async embed(text: string): Promise<Float32Array> {
        state.embedCalls.push(text);
        return buildVector(text);
      },
      async embedMany(texts: string[]): Promise<Float32Array[]> {
        state.embedManyCalls.push([...texts]);
        return texts.map((text) => buildVector(text));
      },
    },
    warnings: [],
  });
}
