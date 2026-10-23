import { test as base, expect } from "@playwright/test";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { browserArtifact } from "../../../../scripts/browser-fixture.mjs";

const artifact = browserArtifact();

export const test = base.extend<{ verifiedCandidate: void }>({
  verifiedCandidate: [
    async ({ request }, use) => {
      const response = await request.get("/");
      expect(response.ok(), "Device fixture must serve the selected Atlas build").toBe(
        true,
      );
      const served = await response.text();
      const local = readFileSync(
        artifact?.index ?? resolve(import.meta.dirname, "../../dist/index.html"),
        "utf8",
      );
      const digest = (value: string) =>
        createHash("sha256").update(value).digest("hex");
      expect(
        digest(served),
        "Device fixture is serving another build; use a separate ATLAS_DEVICE_PORT for each active worktree",
      ).toBe(digest(local));
      await use();
    },
    { auto: true },
  ],
});
export { expect };
