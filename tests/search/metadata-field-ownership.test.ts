import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { SEARCH_METADATA_FIELD_CATALOG } from "../../src/domain/metadata-field-catalog.js";
import { METADATA_ROW_PROJECTIONS } from "../../src/data/metadata-row-projection.js";
import { METADATA_FIELD_EXECUTION_SPECS } from "../../src/search/filters/metadata-execution.js";
import { getMetadataFieldPresentationSpecs } from "../../src/server/metadata-presentation.js";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function listFiles(directory: string): string[] {
  return readdirSync(directory).flatMap((entry) => {
    const entryPath = path.join(directory, entry);
    return statSync(entryPath).isDirectory() ? listFiles(entryPath) : [entryPath];
  });
}

describe("metadata field ownership", () => {
  it("keeps execution, row projection, and presentation specs aligned with the public catalog", () => {
    const catalogFields = new Set(SEARCH_METADATA_FIELD_CATALOG.map((entry) => entry.field));
    const executionFields = new Set(METADATA_FIELD_EXECUTION_SPECS.map((entry) => entry.field));
    const rowProjectionFields = new Set(METADATA_ROW_PROJECTIONS.map((entry) => entry.field));
    const presentationFields = new Set(
      ["summary", "detail"].flatMap((presentation) =>
        getMetadataFieldPresentationSpecs(presentation).map((entry) => entry.field),
      ),
    );

    expect(executionFields).toEqual(catalogFields);
    expect(rowProjectionFields).toEqual(catalogFields);
    expect(presentationFields).toEqual(catalogFields);
  });

  it("removes the old fused metadata registry and semantics modules", () => {
    expect(existsSync(path.join(REPO_ROOT, "src/search/filters/registry.ts"))).toBe(false);
    expect(existsSync(path.join(REPO_ROOT, "src/search/filters/semantics.ts"))).toBe(false);
  });

  it("keeps physical metadata query construction out of search filters", () => {
    const searchFilterFiles = listFiles(path.join(REPO_ROOT, "src/search/filters")).filter((file) =>
      file.endsWith(".ts"),
    );
    const sqlTerms = /buildSqlExpression|SELECT |JOIN |WHERE | EXISTS |json_each|SQL|clause/;
    const offenders = searchFilterFiles
      .map((file) => ({
        file: path.relative(REPO_ROOT, file),
        matches: readFileSync(file, "utf8")
          .split("\n")
          .map((line, index) => ({ index: index + 1, line }))
          .filter(({ line }) => sqlTerms.test(line)),
      }))
      .filter(({ matches }) => matches.length > 0);

    expect(offenders).toEqual([]);
  });

  it("keeps SQLite FTS syntax out of search modules", () => {
    const searchFiles = listFiles(path.join(REPO_ROOT, "src/search")).filter((file) => file.endsWith(".ts"));
    const ftsTerms = /ftsQuery|buildFtsQuery|\bMATCH\b|records_fts|"\$"|\*"\)/;
    const offenders = searchFiles
      .map((file) => ({
        file: path.relative(REPO_ROOT, file),
        matches: readFileSync(file, "utf8")
          .split("\n")
          .map((line, index) => ({ index: index + 1, line }))
          .filter(({ line }) => ftsTerms.test(line)),
      }))
      .filter(({ matches }) => matches.length > 0);

    expect(offenders).toEqual([]);
  });
});
