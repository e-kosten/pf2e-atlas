import { describe, expect, it } from "vitest";

import { loadConfig } from "../src/config.js";

describe("loadConfig", () => {
  it("prefers the CLI data path", async () => {
    const config = await loadConfig(
      ["--data-path", "/Users/ekosten/projects/pathfinder-mcp/pf2e"],
      { PF2E_DATA_PATH: "/tmp/ignored" },
    );

    expect(config.rootPath).toBe("/Users/ekosten/projects/pathfinder-mcp/pf2e");
    expect(config.manifestPath.endsWith("system.pf2e.json") || config.manifestPath.endsWith("static/system.json")).toBe(
      true,
    );
  });

  it("throws when no data path is configured", async () => {
    await expect(loadConfig([], {})).rejects.toThrow(/PF2E data path is required/);
  });
});
