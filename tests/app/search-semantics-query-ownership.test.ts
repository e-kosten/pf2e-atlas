import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const searchSemanticsRoot = path.join(repoRoot, "src/app/ontology/search-semantics");
const queryBuildersPath = path.join(searchSemanticsRoot, "query-builders.ts");

function listSourceFiles(directory: string): string[] {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const absolutePath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      return listSourceFiles(absolutePath);
    }
    return entry.isFile() && entry.name.endsWith(".ts") ? [absolutePath] : [];
  });
}

function relativePath(absolutePath: string): string {
  return path.relative(repoRoot, absolutePath);
}

describe("search-semantics query ownership", () => {
  it("keeps canonical query and filter construction in query builders", () => {
    const queryConstructionPatterns = [
      /\bbuildSearchSemanticsMetadataQuery\b/,
      /\bbuildValueScopedQuery\b/,
      /\bbuildMetadataValueQuery\b/,
      /\bbuildMetricScalarMetadataQuery\b/,
      /\bbuildScopeFilter\b/,
      /mode:\s*"browse"/,
      /kind:\s*"metadataPredicate"/,
      /kind:\s*"metricCompare"/,
      /kind:\s*"pack"/,
    ];

    const violations = listSourceFiles(searchSemanticsRoot)
      .filter((file) => file !== queryBuildersPath)
      .flatMap((file) => {
        const source = fs.readFileSync(file, "utf8");
        return queryConstructionPatterns
          .filter((pattern) => pattern.test(source))
          .map((pattern) => `${relativePath(file)} contains query-builder-only pattern ${pattern}`);
      });

    expect(violations).toEqual([]);
  });
});
