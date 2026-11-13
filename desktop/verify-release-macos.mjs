import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const [plist, configPath, host, cli] = process.argv.slice(2);
const { version, bundle: { macOS } } = JSON.parse(readFileSync(configPath, "utf8"));
function readPlist(path) {
  return JSON.parse(execFileSync("plutil", ["-convert", "json", "-o", "-", path], { encoding: "utf8" }));
}
const actual = readPlist(plist);
const identity = readPlist(macOS.infoPlist);
assert.equal(actual.CFBundleIdentifier, "com.ekosten.pf2e-atlas");
assert.equal(actual.CFBundleShortVersionString, version);
assert.equal(actual.CFBundleVersion, macOS.bundleVersion);
assert.equal(actual.AtlasReleaseTag, identity.AtlasReleaseTag);
assert.equal(actual.AtlasSourceCommit, identity.AtlasSourceCommit);
assert.equal(actual.LSMinimumSystemVersion, macOS.minimumSystemVersion);
assert.ok(actual.CFBundleURLTypes.some((type) => type.CFBundleURLSchemes.includes("pf2e-atlas")));
for (const executable of [host, cli]) {
  const build = execFileSync("vtool", ["-show-build", executable], { encoding: "utf8" });
  const minimum = /^\s*minos\s+(\d+(?:\.\d+){0,2})\s*$/m.exec(build)?.[1];
  assert.ok(minimum, `Missing minimum macOS in ${executable}`);
  const value = (version) => version.split(".").map(Number).reduce((sum, part, index) => sum + part * 1000 ** (2 - index), 0);
  assert.ok(value(minimum) <= value(macOS.minimumSystemVersion), `${executable} requires macOS ${minimum}`);
}
