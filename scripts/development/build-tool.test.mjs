import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  readFileSync,
  readdirSync,
} from "node:fs";
import { resolve, dirname } from "node:path";
import { fixture } from "./test-fixtures.mjs";
import { digest } from "./preflight.mjs";

for (const mode of [
  "publish",
  "source-drift",
  "build-failure",
  "bad-capabilities",
])
  test(`retained-tool builder: ${mode}`, (t) => {
    const f = fixture(t);
    for (const file of [
      "scripts/development/build-tool.mjs",
      "scripts/development/preflight.mjs",
      "scripts/validation/run-check.mjs",
      "scripts/validation/run-process.mjs",
    ]) {
      const destination = resolve(f.root, file);
      mkdirSync(dirname(destination), { recursive: true });
      copyFileSync(new URL("../../" + file, import.meta.url), destination);
    }
    const metadata = readFileSync(resolve(f.root, "bin/cargo"), "utf8")
      .split("\n")
      .slice(1)
      .join("\n");
    f.program(
      "cargo",
      `if [ "$1" = metadata ]; then\n${metadata}\nelif [ "$1" = --version ]; then echo cargo-fixture\nelse\n${mode === "build-failure" ? "exit 31" : "cp bin/atlas-dev target/debug/atlas-dev"}\n${mode === "source-drift" ? "printf changed > crates/atlas-dev/src/main.rs" : ""}\nfi`,
    );
    const destination = f.put(
      ".atlas/dev-baselines/tools/atlas-dev",
      "previous tool",
    );
    f.put(".atlas/dev-baselines/tools/atlas-dev.json", "previous receipt");
    mkdirSync(resolve(f.root, "target/debug"), { recursive: true });
    if (mode === "bad-capabilities") f.program("atlas-dev", "exit 2");
    const override = readFileSync(f.tool);
    const result = spawnSync(
      process.execPath,
      ["scripts/development/build-tool.mjs"],
      {
        cwd: f.root,
        env: process.env,
        encoding: "utf8",
        timeout: 10000,
      },
    );
    assert.equal(
      result.status,
      mode === "publish"
        ? 0
        : mode === "source-drift"
          ? 78
          : mode === "build-failure"
            ? 31
            : 1,
      result.stderr,
    );
    assert.deepEqual(
      readFileSync(f.tool),
      override,
      "runtime override must not become a publication destination",
    );
    if (mode === "publish") {
      const receipt = JSON.parse(readFileSync(destination + ".json"));
      assert.equal(receipt.binary_sha256, digest(readFileSync(destination)));
      assert.equal(receipt.capabilities.workspace_schema_version, 22);
      assert.match(receipt.toolchain, /1.95.0/);
    } else {
      assert.equal(readFileSync(destination, "utf8"), "previous tool");
      assert.equal(
        readFileSync(destination + ".json", "utf8"),
        "previous receipt",
      );
    }
    assert.deepEqual(readdirSync(dirname(destination)).sort(), [
      "atlas-dev",
      "atlas-dev.json",
    ]);
  });
