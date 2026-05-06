import { describe, expect, it } from "vitest";

import {
  buildCandidateCountQuery,
  buildCandidateKeyQuery,
  buildCandidateQuery,
  buildFilterValueQuery,
} from "../../src/search/sql.js";

const EMPTY_FILTERS = {};

describe("search SQL builders", () => {
  it("uses the normal records table reference when no record-key constraint is present", () => {
    const query = buildFilterValueQuery({ field: "traits", category: "creature" }, EMPTY_FILTERS);

    expect(query.sql).toContain("FROM records r\n");
    expect(query.sql).not.toContain("INDEXED BY sqlite_autoindex_records_1");
  });

  it("forces primary-key record lookup for record-key-constrained filter values", () => {
    const query = buildFilterValueQuery({ field: "traits", category: "creature" }, EMPTY_FILTERS, {
      recordKeys: ["creature:dragon", "creature:wight"],
    });

    expect(query.sql).toContain("FROM records r INDEXED BY sqlite_autoindex_records_1");
    expect(query.sql).toContain("r.record_key IN (?, ?)");
    expect(query.params).toEqual(["creature:dragon", "creature:wight"]);
  });

  it("uses primary-key record lookup for record-key-constrained candidate queries", () => {
    const queries = [
      buildCandidateQuery(EMPTY_FILTERS, false, false, { recordKeys: ["creature:dragon"] }),
      buildCandidateCountQuery(EMPTY_FILTERS, { recordKeys: ["creature:dragon"] }),
      buildCandidateKeyQuery(EMPTY_FILTERS, undefined, { recordKeys: ["creature:dragon"] }),
    ];

    for (const query of queries) {
      expect(query.sql).toContain("FROM records r INDEXED BY sqlite_autoindex_records_1");
      expect(query.sql).toContain("r.record_key IN (?)");
      expect(query.params).toEqual(["creature:dragon"]);
    }
  });
});
