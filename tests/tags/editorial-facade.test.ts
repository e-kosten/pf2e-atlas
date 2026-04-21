import { describe, expect, it } from "vitest";

import * as editorial from "../../src/tags/editorial.js";

describe("editorial facade", () => {
  it("does not re-export app ontology entity-record helpers", () => {
    expect(editorial).not.toHaveProperty("buildOntologyExplorerEntityDetailLines");
    expect(editorial).not.toHaveProperty("buildOntologyExplorerEntityRecordSelectColumns");
    expect(editorial).not.toHaveProperty("buildOntologyExplorerEntitySummary");
    expect(editorial).not.toHaveProperty("mapNormalizedRecordToOntologyExplorerEntityRecord");
    expect(editorial).not.toHaveProperty("mapOntologyExplorerEntityRecordRow");
  });
});
