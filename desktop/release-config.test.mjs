import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { releaseConfig } from "./release-config.mjs";

const sha = "abcdef123456" + "0".repeat(28);

test("nightly, RC and stable tags map to numeric macOS versions", () => {
  for (const tag of ["v1.2.3-nightly.20261113000000.gabcdef123456", "v1.2.3-rc.1", "v1.2.3"]) {
    const config = releaseConfig(tag, sha, "12345678901", "/tmp");
    assert.equal(config.version, "1.2.3");
    assert.equal(config.bundle.macOS.bundleVersion, "12345678901");
    assert.equal(config.bundle.macOS.signingIdentity, "-");
  }
  assert.throws(() => releaseConfig("v1.2.3-nightly.20261113000000.g000000000000", sha, "1", "/tmp"));
  assert.throws(() => releaseConfig("v1.2.3", sha, "attempt.1", "/tmp"));
});

test("the Windows installer template tracks the locked Tauri CLI", () => {
  const read = (file) => readFileSync(new URL(file, import.meta.url), "utf8");
  const version = JSON.parse(read("./package-lock.json")).packages["node_modules/@tauri-apps/cli"].version;
  assert.ok(read("./src-tauri/installer.nsi").startsWith("; Adapted from Tauri CLI " + version + "'s "));
});
