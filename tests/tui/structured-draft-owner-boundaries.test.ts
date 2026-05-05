import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const structuredDraftRoot = path.join(repoRoot, "src/tui/search-screen/structured-draft");

function listSourceFiles(directory: string): string[] {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const absolutePath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      return listSourceFiles(absolutePath);
    }
    return entry.isFile() && /\.(ts|tsx)$/.test(entry.name) ? [absolutePath] : [];
  });
}

function relativeSourcePath(absolutePath: string): string {
  return path.relative(repoRoot, absolutePath);
}

describe("structured draft owner boundaries", () => {
  it("keeps generic insertion-result usage inside the prompt-facing explorer owner", () => {
    const uses = listSourceFiles(structuredDraftRoot)
      .filter((file) => fs.readFileSync(file, "utf8").includes("buildFilterExplorerInsertionResult"))
      .map(relativeSourcePath);

    expect(uses).toEqual([
      "src/tui/search-screen/structured-draft/structured-draft-explorer-actions.ts",
    ]);
  });
});
