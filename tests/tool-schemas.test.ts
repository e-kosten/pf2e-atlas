import { describe, expect, it } from "vitest";

import { listRecordsModeSchema, searchModeSchema } from "../src/tool-schemas.js";

describe("tool schemas", () => {
  it("only advertises structured mode for list-records", () => {
    expect(listRecordsModeSchema.safeParse("structured").success).toBe(true);
    expect(listRecordsModeSchema.safeParse("lexical").success).toBe(false);
    expect(listRecordsModeSchema.safeParse("hybrid").success).toBe(false);
  });

  it("continues to advertise lexical and hybrid search modes where supported", () => {
    expect(searchModeSchema.safeParse("structured").success).toBe(true);
    expect(searchModeSchema.safeParse("lexical").success).toBe(true);
    expect(searchModeSchema.safeParse("hybrid").success).toBe(true);
  });
});
