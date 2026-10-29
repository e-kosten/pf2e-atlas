import test from "node:test";
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { resolve, dirname } from "node:path";
import { fixture } from "./test-fixtures.mjs";
import { digest } from "./preflight.mjs";

function prepare(t, profile = "dev") {
  const f = fixture(t);
  const files = [
    "scripts/development/shared-refresh.mjs",
    "scripts/development/build-profile.mjs",
    "scripts/development/preflight.mjs",
    "scripts/validation/run-check.mjs",
    "scripts/validation/inputs.mjs",
    "scripts/validation/run-process.mjs",
  ];
  for (const file of files) {
    const destination = resolve(f.root, file);
    mkdirSync(dirname(destination), { recursive: true });
    copyFileSync(new URL("../../" + file, import.meta.url), destination);
  }
  const source = resolve(f.root, ".atlas/shared-source");
  const models = resolve(f.root, ".atlas/shared-models");
  mkdirSync(source, { recursive: true });
  mkdirSync(models, { recursive: true });
  const store = resolve(f.root, ".atlas/dev-baselines");
  const log = resolve(f.root, ".atlas/maintenance-log");
  const program = `#!${process.execPath}
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const args=process.argv.slice(2),get=name=>args[args.indexOf(name)+1];
if(args[1]==='capabilities'){console.log(JSON.stringify({version:1,managed_preview:true,workspace_schema_version:22}));process.exit(0);}
if(args[1]==='init'){
 const index=path.join(get('--checkout'),'.atlas/workspace.sqlite');fs.mkdirSync(path.dirname(index),{recursive:true});fs.writeFileSync(index,'controlled seed');
 fs.appendFileSync(process.env.CALLER_SOURCE,'\\n// caller advanced after freeze\\n');
 fs.appendFileSync(process.env.CALLER_RUNNER,'\\n// caller runner advanced after load\\n');
 fs.writeFileSync(process.env.MUTABLE_TOOL,'#!/bin/sh\\nexit 92\\n');
 fs.writeFileSync(process.env.MUTABLE_APP,'#!/bin/sh\\nexit 91\\n');
}else if(args[1]==='publish'){
 fs.appendFileSync(process.env.MAINTENANCE_LOG,JSON.stringify({index:get('--index'),binary:get('--binary'),profile:get('--build-profile'),commit:get('--build-commit')})+'\\n');
 if(process.env.FAIL_PUBLICATION)process.exit(17);
 if(crypto.createHash('sha256').update(fs.readFileSync(get('--binary'))).digest('hex')!==process.env.EXPECTED_APP_SHA)process.exit(92);
 const store=get('--store');fs.mkdirSync(path.join(store,'versions/new'),{recursive:true});fs.copyFileSync(get('--binary'),path.join(store,'versions/new/atlas'));
 fs.writeFileSync(path.join(store,'current.json'),JSON.stringify({version:1,baseline:'new'}));
}else process.exit(93);
`;
  writeFileSync(f.tool, program);
  chmodSync(f.tool, 0o755);
  f.publish();
  f.put(
    ".atlas/dev-baselines/versions/seed/manifest.json",
    JSON.stringify({
      database: { schema_version: 19 },
      assets: { source, embedding_cache: models },
    }),
  );
  const app = f.put(
    profile === "release" ? "target/release/atlas" : "target/debug/atlas",
    `#!${process.execPath}
const fs=require('node:fs'),args=process.argv.slice(2);if(args[0]!=='setup')process.exit(94);const index=args[args.indexOf('--index')+1];fs.writeFileSync(index,'upgraded controlled seed22');
`,
  );
  chmodSync(app, 0o755);
  execFileSync("git", ["add", "."], { cwd: f.root });
  execFileSync("git", ["commit", "-qm", "test: maintenance candidate"], {
    cwd: f.root,
  });
  const commit = execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: f.root,
    encoding: "utf8",
  }).trim();
  const sha = digest(readFileSync(app));
  f.put(
    profile === "release"
      ? ".atlas/maintenance-build.json"
      : ".atlas/preview-build.json",
    JSON.stringify({
      version: 1,
      profile,
      source: { dirty: false, commit },
      binary_sha256: sha,
      toolchain: "rustc 1.95.0 (fixture)",
    }),
  );
  return { ...f, store, log, app, sha, profile, commit };
}

for (const profile of ["dev", "release"])
  for (const fail of [false, true])
    test(`controlled ${profile} seed refresh ${fail ? "failure preserves pointer" : "uses frozen verified application"}`, (t) => {
      const f = prepare(t, profile);
      const task = readFileSync(resolve(f.root, ".atlas/workspace.sqlite"));
      const result = spawnSync(
        process.execPath,
        [
          "scripts/development/shared-refresh.mjs",
          ...(profile === "dev" ? ["--profile", "dev"] : []),
        ],
        {
          cwd: f.root,
          env: {
            ...process.env,
            MAINTENANCE_LOG: f.log,
            MUTABLE_APP: f.app,
            MUTABLE_TOOL: f.tool,
            CALLER_SOURCE: resolve(f.root, "crates/atlas-dev/src/main.rs"),
            CALLER_RUNNER: resolve(f.root, "scripts/validation/run-check.mjs"),
            EXPECTED_APP_SHA: f.sha,
            ...(fail ? { FAIL_PUBLICATION: "1" } : {}),
          },
          encoding: "utf8",
          timeout: 10000,
        },
      );
      assert.equal(result.status, fail ? 17 : 0, result.stderr);
      assert.deepEqual(
        readFileSync(resolve(f.root, ".atlas/workspace.sqlite")),
        task,
      );
      assert.equal(
        JSON.parse(readFileSync(resolve(f.store, "current.json"))).baseline,
        fail ? "seed" : "new",
      );
      const entry = JSON.parse(readFileSync(f.log, "utf8").trim());
      assert.notEqual(entry.binary, f.app);
      assert.equal(entry.profile, profile);
      assert.equal(entry.commit, f.commit);
      assert.match(
        readFileSync(resolve(f.root, "crates/atlas-dev/src/main.rs"), "utf8"),
        /caller advanced/,
      );
      assert.notEqual(entry.index, resolve(f.root, ".atlas/workspace.sqlite"));
      assert.equal(existsSync(entry.index), false);
      if (!fail)
        assert.equal(
          digest(readFileSync(resolve(f.store, "versions/new/atlas"))),
          f.sha,
        );
    });

test("maintenance defaults to release and never silently publishes the available dev build", (t) => {
  const f = prepare(t);
  const result = spawnSync(
    process.execPath,
    ["scripts/development/shared-refresh.mjs"],
    { cwd: f.root, env: process.env, encoding: "utf8" },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /release build receipt missing/);
  assert.match(result.stderr, /dev-build --profile release/);
  assert.equal(
    JSON.parse(readFileSync(resolve(f.store, "current.json"))).baseline,
    "seed",
  );
  assert.equal(existsSync(f.log), false);
});

test("maintenance rejects a receipt for another build profile", (t) => {
  const f = prepare(t, "release");
  const path = resolve(f.root, ".atlas/maintenance-build.json");
  const receipt = JSON.parse(readFileSync(path, "utf8"));
  receipt.profile = "dev";
  writeFileSync(path, JSON.stringify(receipt));
  const result = spawnSync(
    process.execPath,
    ["scripts/development/shared-refresh.mjs"],
    { cwd: f.root, env: process.env, encoding: "utf8" },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /clean release build provenance required/);
  assert.equal(
    JSON.parse(readFileSync(resolve(f.store, "current.json"))).baseline,
    "seed",
  );
  assert.equal(existsSync(f.log), false);
});
