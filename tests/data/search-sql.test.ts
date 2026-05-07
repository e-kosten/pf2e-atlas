import { DatabaseSync } from "node:sqlite";

import { describe, expect, it } from "vitest";

import {
  buildCandidateCountQuery,
  buildCandidateKeyQuery,
  buildCandidateQuery,
  buildFilterValueQuery,
} from "../../src/data/backend/search-sql.js";
import { fetchSemanticRetrievalRows } from "../../src/data/record-queries.js";

const EMPTY_FILTERS = {};

function createSearchSqlFixture(): DatabaseSync {
  const db = new DatabaseSync(":memory:");
  db.exec(`
    CREATE TABLE records (
      record_key TEXT PRIMARY KEY,
      is_search_canonical INTEGER NOT NULL,
      category TEXT NOT NULL,
      subcategory TEXT,
      pack_name TEXT,
      level INTEGER,
      rarity TEXT
    );
    CREATE TABLE actor_records (record_key TEXT PRIMARY KEY);
    CREATE TABLE item_records (record_key TEXT PRIMARY KEY);
    CREATE TABLE spell_records (record_key TEXT PRIMARY KEY);
    CREATE TABLE record_traits (record_key TEXT NOT NULL, trait TEXT NOT NULL);
    CREATE TABLE actor_metrics (
      record_key TEXT NOT NULL,
      metric_key TEXT NOT NULL,
      value_type TEXT NOT NULL,
      number_value REAL,
      text_value TEXT,
      bool_value INTEGER
    );
  `);
  const insertRecord = db.prepare(
    "INSERT INTO records (record_key, is_search_canonical, category, subcategory, pack_name, level, rarity) VALUES (?, ?, ?, ?, ?, ?, ?)",
  );
  insertRecord.run("creature:dragon", 1, "creature", null, "bestiary", 10, "common");
  insertRecord.run("creature:wight", 1, "creature", null, "bestiary", 3, "common");
  insertRecord.run("creature:ghost", 1, "creature", null, "bestiary", 5, "rare");
  db.exec(`
    INSERT INTO actor_records (record_key) VALUES ('creature:dragon'), ('creature:wight'), ('creature:ghost');
    INSERT INTO record_traits (record_key, trait) VALUES
      ('creature:dragon', 'dragon'),
      ('creature:dragon', 'fire'),
      ('creature:wight', 'undead'),
      ('creature:ghost', 'undead');
    INSERT INTO actor_metrics (record_key, metric_key, value_type, number_value, text_value, bool_value) VALUES
      ('creature:dragon', 'save.best', 'text', NULL, 'fort', NULL),
      ('creature:wight', 'save.best', 'text', NULL, 'will', NULL),
      ('creature:ghost', 'save.best', 'text', NULL, 'will', NULL);
  `);
  return db;
}

function executeRows(db: DatabaseSync, query: { sql: string; params: unknown[] }): unknown[] {
  return db.prepare(query.sql).all(...query.params);
}

function withoutPrimaryKeyHint(query: { sql: string; params: unknown[] }): { sql: string; params: unknown[] } {
  return {
    ...query,
    sql: query.sql.replace(" INDEXED BY sqlite_autoindex_records_1", ""),
  };
}

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

  it("preserves record-key-constrained trait and metric value counts when forcing primary-key lookup", () => {
    const db = createSearchSqlFixture();
    try {
      const recordKeys = ["creature:dragon", "creature:wight"];
      const traitQuery = buildFilterValueQuery({ field: "traits", category: "creature" }, EMPTY_FILTERS, {
        recordKeys,
      });
      const metricQuery = buildFilterValueQuery(
        { field: "actorMetrics", category: "creature", metric: "save.best" },
        EMPTY_FILTERS,
        { recordKeys },
      );

      expect(executeRows(db, traitQuery)).toEqual(executeRows(db, withoutPrimaryKeyHint(traitQuery)));
      expect(executeRows(db, metricQuery)).toEqual(executeRows(db, withoutPrimaryKeyHint(metricQuery)));
    } finally {
      db.close();
    }
  });

  it("widens filtered semantic retrieval inside the data-owned retrieval path", () => {
    let preparedSql = "";
    const db = {
      prepare: (sql: string) => {
        preparedSql = sql;
        return { all: () => [] };
      },
    } as unknown as DatabaseSync;

    fetchSemanticRetrievalRows(
      db,
      {
        filter: {
          kind: "scope",
          category: "creature",
          subcategory: { kind: "any" },
        },
      },
      new Float32Array([1, 2, 3]),
      80,
    );

    expect(preparedSql).toContain("AND k = 160");
  });
});
