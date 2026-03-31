import { describe, expect, it } from "vitest";

import { searchProfileSchema } from "../src/tool-schemas.js";

describe("tool schemas", () => {
  it("advertises the user-facing search profiles", () => {
    expect(searchProfileSchema.safeParse("lookup").success).toBe(true);
    expect(searchProfileSchema.safeParse("balanced").success).toBe(true);
    expect(searchProfileSchema.safeParse("concept").success).toBe(true);
    expect(searchProfileSchema.safeParse("semantic_only").success).toBe(false);
  });
});
