import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function listSourceFiles(directory: string): string[] {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const absolutePath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      return listSourceFiles(absolutePath);
    }
    return entry.isFile() && /\.(ts|tsx)$/.test(entry.name) ? [absolutePath] : [];
  });
}

function relativePath(absolutePath: string): string {
  return path.relative(repoRoot, absolutePath);
}

function legacyTokenParts(): string[][] {
  return [
    ["metadata", "-filter", "-draft"],
    ["Metadata", "Filter", "Node"],
    ["metadata", "Filter", "Node", "To", "Canonical", "Filter"],
    ["canonical", "Filter", "To", "Metadata", "Node"],
  ];
}

describe("canonical search state boundaries", () => {
  it("keeps TUI search editing on canonical filter trees", () => {
    const files = [
      ...listSourceFiles(path.join(repoRoot, "src/tui")),
      ...listSourceFiles(path.join(repoRoot, "tests")),
    ];
    const legacyTokens = legacyTokenParts().map((parts) => parts.join(""));
    const structuralTreePattern = /\|\s*\{\s*(and|or|not)\s*:/;
    const localSemanticNamePattern = /(?:type|interface)\s+\w*Metadata\w*(?:Filter|Predicate)\w*(?:Ast|Node|Tree)\b/i;
    const violations = files.flatMap((file) => {
      const source = fs.readFileSync(file, "utf8");
      const tokenViolations = legacyTokens
        .filter((token) => source.includes(token))
        .map((token) => `${relativePath(file)} contains ${token}`);
      const shapeViolations = structuralTreePattern.test(source)
        ? [`${relativePath(file)} declares non-canonical and/or/not filter branches`]
        : [];
      const namingViolations = localSemanticNamePattern.test(source)
        ? [`${relativePath(file)} declares a TUI-local semantic metadata filter type`]
        : [];
      return [...tokenViolations, ...shapeViolations, ...namingViolations];
    });

    expect(violations).toEqual([]);
  });
});
