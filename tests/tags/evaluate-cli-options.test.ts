import { describe, expect, it } from "vitest";

import { parseOptions as parseGapOptions } from "../../src/tags/cli/evaluation/evaluate-gaps.js";
import { parseOptions as parseMovementOptions } from "../../src/tags/cli/evaluation/evaluate-movement.js";

describe("evaluation CLI option parsing", () => {
  it("rejects invalid gap evaluation category scopes", () => {
    expect(() => parseGapOptions(["--tag", "urban_setting", "--category", "relic"])).toThrow(
      /Invalid search category/i,
    );
    expect(() =>
      parseGapOptions(["--tag", "urban_setting", "--category", "creature", "--subcategory", "gear"]),
    ).toThrow(/Invalid search subcategory/i);
    expect(() =>
      parseGapOptions([
        "--tag",
        "urban_setting",
        "--exemplar-category",
        "equipment",
        "--exemplar-subcategory",
        "character",
      ]),
    ).toThrow(/Invalid search subcategory/i);
  });

  it("rejects invalid movement evaluation categories", () => {
    expect(() => parseMovementOptions(["--baseline-index-path", "./baseline.sqlite", "--category", "relic"])).toThrow(
      /Invalid search category/i,
    );
  });
});
