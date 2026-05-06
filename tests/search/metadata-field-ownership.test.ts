import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { describe, expect, it } from "vitest";

import { SEARCH_METADATA_FIELD_CATALOG } from "../../src/domain/metadata-field-catalog.js";
import { METADATA_ROW_PROJECTIONS } from "../../src/data/metadata-row-projection.js";
import { METADATA_FIELD_EXECUTION_SPECS } from "../../src/search/filters/metadata-execution.js";
import { getMetadataFieldPresentationSpecs } from "../../src/server/metadata-presentation.js";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

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
});
