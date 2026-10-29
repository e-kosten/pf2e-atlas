import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const scripts = dirname(fileURLToPath(import.meta.url));
function fixture(t) {
  const root = realpathSync(
    mkdtempSync(resolve(tmpdir(), "atlas-preview profile-")),
  );
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const put = (name, text) => {
    const path = resolve(root, name);
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, text);
  };
  execFileSync("git", ["init", "-q", root]);
  for (const name of ["preview.sh", "build-profile.mjs"])
    put(`scripts/development/${name}`, readFileSync(resolve(scripts, name)));
  put(
    "scripts/development/record.mjs",
    `
import { appendFileSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
const [step,...args]=process.argv.slice(2);
const root=${JSON.stringify(root)};
appendFileSync(resolve(root,'calls.jsonl'),JSON.stringify({step,args})+'\\n');
if(process.env.ATLAS_PREVIEW_TEST_FAIL===step)process.exit(7);
if(step==='build'){
 const profile=args[args.indexOf('--profile')+1];
 const binary=resolve(root,'target',profile==='release'?'release':'debug',process.platform==='win32'?'atlas.exe':'atlas');
 const receipt=resolve(root,'.atlas',profile==='release'?'maintenance-build.json':'preview-build.json');
 for(const path of [binary,receipt])mkdirSync(dirname(path),{recursive:true});
 writeFileSync(binary,profile);
 writeFileSync(receipt,JSON.stringify({profile}));
}
if(step==='launch'){
 const binary=args[args.indexOf('--binary')+1];
 const receipt=JSON.parse(readFileSync(args[args.indexOf('--build-provenance')+1],'utf8'));
 if(readFileSync(binary,'utf8')!==receipt.profile)throw new Error('Selected binary/receipt mismatch');
 writeFileSync(resolve(root,'server-state'),'replacement');
}
`,
  );
  put(
    "scripts/development/preflight.mjs",
    `process.argv.splice(2,0,'preflight'); await import('./record.mjs');\n`,
  );
  put(
    "scripts/development/build-preview.mjs",
    `process.argv.splice(2,0,'build'); await import('./record.mjs');\n`,
  );
  put(
    "scripts/development/atlas-dev.sh",
    '#!/bin/sh\nexec node "$(dirname "$0")/record.mjs" launch "$@"\n',
  );
  put("server-state", "original");
  return {
    root,
    run(mode, args = [], fail) {
      return spawnSync(
        "sh",
        [resolve(root, "scripts/development/preview.sh"), mode, root, ...args],
        {
          cwd: root,
          env: { ...process.env, ATLAS_PREVIEW_TEST_FAIL: fail ?? "" },
          encoding: "utf8",
        },
      );
    },
    calls() {
      const path = resolve(root, "calls.jsonl");
      return existsSync(path)
        ? readFileSync(path, "utf8").trim().split("\n").map(JSON.parse)
        : [];
    },
    state: () => readFileSync(resolve(root, "server-state"), "utf8"),
  };
}

for (const [mode, options, profile, directory, receipt] of [
  ["start", [], "dev", "debug", "preview-build.json"],
  [
    "start",
    ["--profile", "release"],
    "release",
    "release",
    "maintenance-build.json",
  ],
  [
    "refresh",
    ["--profile=release"],
    "release",
    "release",
    "maintenance-build.json",
  ],
  ["refresh", ["--profile", "dev"], "dev", "debug", "preview-build.json"],
]) {
  test(`${mode} ${profile} selects the built binary and matching receipt`, (t) => {
    const f = fixture(t);
    const forwarded = mode === "start" ? ["--port", "4827"] : ["--json"];
    const result = f.run(mode, [...options, ...forwarded]);
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(f.calls(), [
      { step: "preflight", args: ["--preview", f.root] },
      { step: "build", args: [f.root, "--profile", profile] },
      {
        step: "launch",
        args: [
          ...(mode === "start" ? ["web", "--managed"] : ["refresh"]),
          ...forwarded,
          "--checkout",
          f.root,
          "--binary",
          resolve(
            f.root,
            "target",
            directory,
            process.platform === "win32" ? "atlas.exe" : "atlas",
          ),
          "--frontend-index",
          resolve(f.root, "web/atlas-ui/dist/index.html"),
          "--build-provenance",
          resolve(f.root, ".atlas", receipt),
        ],
      },
    ]);
    assert.equal(f.state(), "replacement");
  });
}

for (const args of [
  ["--profile", "fast"],
  ["--profile"],
  ["--profile="],
  ["--profile", "release", "--profile", "dev"],
]) {
  test(`invalid profile selection ${JSON.stringify(args)} preserves the running preview`, (t) => {
    const f = fixture(t);
    const result = f.run("refresh", args);
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /profile/);
    assert.deepEqual(f.calls(), []);
    assert.equal(f.state(), "original");
  });
}
for (const fail of ["preflight", "build"]) {
  test(`${fail} failure never reaches the managed server`, (t) => {
    const f = fixture(t);
    const result = f.run("refresh", ["--profile", "release"], fail);
    assert.equal(result.status, 7, result.stderr);
    assert(!f.calls().some((call) => call.step === "launch"));
    assert.equal(f.state(), "original");
  });
}
