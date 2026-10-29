import { test } from "node:test";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { resolve } from "node:path";
import {
  frontendInputIdentity,
  readAtlasApiContract,
} from "../web/atlas-ui/apiContract.ts";

function fixture() {
  const directory = mkdtempSync(resolve(tmpdir(), "atlas-isolated-browser-"));
  const root = resolve(directory, "source");
  mkdirSync(resolve(root, "scripts"), { recursive: true });
  mkdirSync(resolve(root, "web/atlas-ui"), { recursive: true });
  mkdirSync(resolve(root, "crates/atlas-web"), { recursive: true });
  mkdirSync(resolve(root, "crates/atlas-app-model/bindings"), {
    recursive: true,
  });
  copyFileSync(
    new URL("../web/atlas-ui/apiContract.ts", import.meta.url),
    resolve(root, "web/atlas-ui/apiContract.ts"),
  );
  writeFileSync(
    resolve(root, "crates/atlas-web/api-contract.json"),
    JSON.stringify({
      protocol_revision: 1,
      bindings: "crates/atlas-app-model/bindings",
    }),
  );
  writeFileSync(
    resolve(root, "crates/atlas-app-model/bindings/Test.ts"),
    "export type Test = string;\n",
  );
  for (const name of ["browser-fixture.mjs", "validate-browser-candidate.mjs"])
    copyFileSync(
      new URL(name, import.meta.url),
      resolve(root, "scripts", name),
    );
  writeFileSync(
    resolve(root, "web/atlas-ui/package-lock.json"),
    '{"lockfileVersion":3}\n',
  );
  writeFileSync(
    resolve(root, "web/atlas-ui/playwright.devices.config.ts"),
    "candidate A config\n",
  );
  writeFileSync(resolve(root, ".gitignore"), "bin/\nnode_modules/\n");
  const git = (...args) =>
    execFileSync("git", args, {
      cwd: root,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  git("init", "-q");
  git("config", "user.name", "Atlas fixture");
  git("config", "user.email", "atlas-fixture@example.invalid");
  git("config", "commit.gpgsign", "false");
  git("config", "core.hooksPath", "/dev/null");
  git("add", ".");
  git("commit", "-qm", "test: candidate A");
  const commit = git("rev-parse", "HEAD");
  const artifact = resolve(directory, "artifact");
  mkdirSync(artifact);
  writeFileSync(resolve(artifact, "browser_fixture"), "#!/bin/sh\nexit 0\n");
  chmodSync(resolve(artifact, "browser_fixture"), 0o755);
  writeFileSync(resolve(artifact, "index.html"), "candidate A frontend");
  const sha256 = Object.fromEntries(
    ["browser_fixture", "index.html"].map((name) => [
      name,
      createHash("sha256")
        .update(readFileSync(resolve(artifact, name)))
        .digest("hex"),
    ]),
  );
  writeFileSync(
    resolve(artifact, "manifest.json"),
    JSON.stringify({
      version: 2,
      commit,
      frontend: {
        input_sha256: frontendInputIdentity(root),
        bundle_sha256: "1".repeat(64),
        api_contract: "fixture-contract",
      },
      platform: process.platform,
      arch: process.arch,
      sha256,
    }),
  );
  mkdirSync(resolve(root, "bin"));
  const npm = resolve(root, "bin/npm");
  writeFileSync(
    npm,
    `#!${process.execPath}
const fs=require('node:fs'); const cp=require('node:child_process'); const path=require('node:path');
const source=process.env.ATLAS_TEST_SOURCE;
if(process.argv[2]==='ci') {
  if(fs.existsSync('node_modules')) throw new Error('Dependencies must be private and initially absent');
  fs.mkdirSync('node_modules/@playwright/test',{recursive:true});
  fs.writeFileSync('node_modules/@playwright/test/package.json',JSON.stringify({version:'fixture-playwright'}));
} else if(process.argv[2]==='exec') {
  if(process.env.ATLAS_TEST_FAIL) process.exit(7);
  if(process.env.ATLAS_TEST_INTERRUPT) {
    fs.writeFileSync(process.env.ATLAS_TEST_CHILD_PID_FILE,String(process.pid));
    process.kill(process.ppid,'SIGTERM');
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)),0,0,10000);
    process.exit(99);
  }
  fs.writeFileSync(path.join(source,'web/atlas-ui/playwright.devices.config.ts'),'candidate B config\\n');
  cp.execFileSync('git',['add','.'],{cwd:source});
  cp.execFileSync('git',['commit','-qm','test: candidate B'],{cwd:source});
  fs.writeFileSync(path.join(source,'web/atlas-ui/package-lock.json'),'working dependency edit\\n');
  if(fs.readFileSync('playwright.devices.config.ts','utf8')!=='candidate A config\\n')throw new Error('Mixed test configuration');
  if(fs.readFileSync('package-lock.json','utf8')!=='{"lockfileVersion":3}\\n')throw new Error('Mixed dependencies');
  if(process.env.ATLAS_TEST_MUTATE_ISOLATED) {
    fs.writeFileSync('unexpected-source.ts','untracked source\\n');
    if(process.env.ATLAS_TEST_MUTATE_ISOLATED==='commit') {
      const cwd=path.resolve('../..');
      cp.execFileSync('git',['add','.'],{cwd});
      cp.execFileSync('git',['-c','user.name=Fixture','-c','user.email=fixture@example.invalid','-c','commit.gpgsign=false','-c','core.hooksPath=/dev/null','commit','-qm','test: mutate isolated source'],{cwd});
    }
    process.exit(0);
  }
  import(path.join(process.cwd(),'../../scripts/browser-fixture.mjs')).then(({browserFixture})=> {
    const fixture=browserFixture();
    if(fs.readFileSync(fixture.index,'utf8')!=='candidate A frontend')throw new Error('Mixed application');
  }).catch(error=>{console.error(error);process.exitCode=1;});
} else { throw new Error('Unexpected npm command'); }
`,
  );
  chmodSync(npm, 0o755);
  const env = {
    ...process.env,
    PATH: `${resolve(root, "bin")}:${process.env.PATH}`,
    ATLAS_TEST_SOURCE: root,
    ATLAS_DEVICE_PORT: "18899",
  };
  return { directory, root, git, commit, artifact, env };
}

test("final browser run keeps app, tests and dependencies pinned while development advances", () => {
  const f = fixture();
  try {
    const output = resolve(f.directory, "run with spaces");
    const result = spawnSync(
      process.execPath,
      [
        resolve(f.root, "scripts/validate-browser-candidate.mjs"),
        "--artifact",
        f.artifact,
        "--output",
        output,
        "--",
        "--project=phone-webkit",
      ],
      { env: f.env, encoding: "utf8" },
    );
    assert.equal(result.status, 0, result.stdout + result.stderr);
    const report = JSON.parse(readFileSync(resolve(output, "result.json")));
    assert.equal(report.status, "passed");
    assert.equal(report.commit, f.commit);
    assert.notEqual(f.git("rev-parse", "HEAD"), f.commit);
    assert.equal(report.playwright, "fixture-playwright");
    assert.equal(report.stages.at(-1).status, "passed");
    assert.equal(existsSync(resolve(output, "checkout")), false);
    assert.equal(
      readFileSync(resolve(f.root, "web/atlas-ui/package-lock.json"), "utf8"),
      "working dependency edit\n",
    );
    assert.equal(existsSync(resolve(output, "artifact/browser_fixture")), true);
    assert.equal(existsSync(resolve(output, "browsers.log")), true);
    assert.equal(
      report.stages.some((s) => s.command[0] === "cargo"),
      false,
    );
  } finally {
    rmSync(f.directory, { recursive: true, force: true });
  }
});

test("candidate mismatch or corrupt application is rejected before dependency installation", () => {
  const f = fixture();
  try {
    writeFileSync(resolve(f.root, "new-file"), "candidate B");
    f.git("add", ".");
    f.git("commit", "-qm", "test: later source");
    const script = resolve(f.root, "scripts/validate-browser-candidate.mjs");
    const output = resolve(f.directory, "mismatched");
    let result = spawnSync(
      process.execPath,
      [
        script,
        "--artifact",
        f.artifact,
        "--commit",
        "HEAD",
        "--output",
        output,
      ],
      { env: f.env, encoding: "utf8" },
    );
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /another commit/);
    assert.equal(existsSync(output), false);
    writeFileSync(resolve(f.artifact, "browser_fixture"), "corrupted");
    result = spawnSync(
      process.execPath,
      [script, "--artifact", f.artifact, "--output", output],
      { env: f.env, encoding: "utf8" },
    );
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /checksum/);
    assert.equal(existsSync(output), false);
  } finally {
    rmSync(f.directory, { recursive: true, force: true });
  }
});

test("failed browser run retains candidate-bound diagnostics and removes private dependencies", () => {
  const f = fixture();
  try {
    const output = resolve(f.directory, "failure");
    const result = spawnSync(
      process.execPath,
      [
        resolve(f.root, "scripts/validate-browser-candidate.mjs"),
        "--artifact",
        f.artifact,
        "--output",
        output,
      ],
      { env: { ...f.env, ATLAS_TEST_FAIL: "1" }, encoding: "utf8" },
    );
    assert.notEqual(result.status, 0);
    const report = JSON.parse(readFileSync(resolve(output, "result.json")));
    assert.equal(report.status, "failed");
    assert.equal(report.commit, f.commit);
    assert.equal(report.stages.at(-1).exit_code, 7);
    assert.equal(existsSync(resolve(output, "browsers.log")), true);
    assert.equal(existsSync(resolve(output, "checkout")), false);
  } finally {
    rmSync(f.directory, { recursive: true, force: true });
  }
});

test("final runner rejects configuration overrides and non-validating browser modes", () => {
  const f = fixture();
  try {
    const script = resolve(f.root, "scripts/validate-browser-candidate.mjs");
    for (const option of [
      "-c/another/config.ts",
      "--config=/another/config.ts",
      "--list",
      "--pass-with-no-tests",
      "--ignore-snapshots",
      "--update-snapshots",
      "--retries=4",
      "--repeat-each=0",
    ]) {
      const output = resolve(f.directory, "rejected");
      const result = spawnSync(
        process.execPath,
        [script, "--artifact", f.artifact, "--output", output, "--", option],
        { env: f.env, encoding: "utf8" },
      );
      assert.notEqual(result.status, 0, option);
      assert.match(
        result.stderr,
        /Unsupported final-validation option|at least one iteration/,
        option,
      );
      assert.equal(existsSync(output), false);
    }
  } finally {
    rmSync(f.directory, { recursive: true, force: true });
  }
});

test("isolated HEAD changes and untracked source fail candidate attribution", () => {
  for (const mode of ["commit", "untracked"]) {
    const f = fixture();
    try {
      const output = resolve(f.directory, mode);
      const result = spawnSync(
        process.execPath,
        [
          resolve(f.root, "scripts/validate-browser-candidate.mjs"),
          "--artifact",
          f.artifact,
          "--output",
          output,
        ],
        {
          env: { ...f.env, ATLAS_TEST_MUTATE_ISOLATED: mode },
          encoding: "utf8",
        },
      );
      assert.notEqual(result.status, 0, mode);
      const report = JSON.parse(readFileSync(resolve(output, "result.json")));
      assert.equal(report.status, "failed");
      assert.match(report.error, /Isolated candidate source changed/, mode);
      assert.equal(report.commit, f.commit);
    } finally {
      rmSync(f.directory, { recursive: true, force: true });
    }
  }
});

test("result records the verified private artifact when a same-commit build is repackaged during copying", () => {
  const f = fixture();
  try {
    const before = JSON.parse(
      readFileSync(resolve(f.artifact, "manifest.json")),
    );
    const patch = resolve(f.directory, "copy-hook.cjs");
    writeFileSync(
      patch,
      `const fs=require('node:fs');const path=require('node:path');const crypto=require('node:crypto');
const original=fs.copyFileSync;let changed=false;
fs.copyFileSync=function(from,to,...options){
 if(!changed&&path.dirname(from)===process.env.ATLAS_TEST_ARTIFACT&&path.basename(from)==='browser_fixture'){
  changed=true;fs.appendFileSync(from,'# another build of same source\\n');
  const manifestPath=path.join(path.dirname(from),'manifest.json');const manifest=JSON.parse(fs.readFileSync(manifestPath));
  manifest.sha256.browser_fixture=crypto.createHash('sha256').update(fs.readFileSync(from)).digest('hex');fs.writeFileSync(manifestPath,JSON.stringify(manifest));
 }
 return original(from,to,...options);
};require('node:module').syncBuiltinESMExports();`,
    );
    const output = resolve(f.directory, "repackaged");
    const result = spawnSync(
      process.execPath,
      [
        resolve(f.root, "scripts/validate-browser-candidate.mjs"),
        "--artifact",
        f.artifact,
        "--output",
        output,
      ],
      {
        env: {
          ...f.env,
          NODE_OPTIONS: `--require=${patch}`,
          ATLAS_TEST_ARTIFACT: f.artifact,
        },
        encoding: "utf8",
      },
    );
    assert.equal(result.status, 0, result.stdout + result.stderr);
    const report = JSON.parse(readFileSync(resolve(output, "result.json")));
    const privateManifest = JSON.parse(
      readFileSync(resolve(output, "artifact/manifest.json")),
    );
    assert.notEqual(
      privateManifest.sha256.browser_fixture,
      before.sha256.browser_fixture,
    );
    assert.deepEqual(report.artifact, privateManifest);
  } finally {
    rmSync(f.directory, { recursive: true, force: true });
  }
});

test("frontend inputs that cannot be reconstructed from the pinned candidate fail clearly", () => {
  const f = fixture();
  try {
    writeFileSync(
      resolve(f.root, "web/atlas-ui/.env.local"),
      "VITE_PREVIEW_SETTING=local-only\n",
    );
    const manifestPath = resolve(f.artifact, "manifest.json");
    const manifest = JSON.parse(readFileSync(manifestPath));
    manifest.frontend.input_sha256 = frontendInputIdentity(f.root);
    writeFileSync(manifestPath, JSON.stringify(manifest));
    const output = resolve(f.directory, "local-env");
    const result = spawnSync(
      process.execPath,
      [
        resolve(f.root, "scripts/validate-browser-candidate.mjs"),
        "--artifact",
        f.artifact,
        "--output",
        output,
      ],
      { env: f.env, encoding: "utf8" },
    );
    assert.notEqual(result.status, 0);
    const report = JSON.parse(readFileSync(resolve(output, "result.json")));
    assert.equal(report.status, "failed");
    assert.match(report.error, /Packaged frontend inputs do not match/);
    assert.equal(
      readFileSync(resolve(f.root, "web/atlas-ui/.env.local"), "utf8"),
      "VITE_PREVIEW_SETTING=local-only\n",
    );
    assert.equal(existsSync(resolve(output, "checkout")), false);
  } finally {
    rmSync(f.directory, { recursive: true, force: true });
  }
});

test("interrupting the runner cleans its owned command and retains interrupted evidence", () => {
  const f = fixture();
  try {
    const output = resolve(f.directory, "interrupt");
    const pidFile = resolve(f.directory, "child.pid");
    const result = spawnSync(
      process.execPath,
      [
        resolve(f.root, "scripts/validate-browser-candidate.mjs"),
        "--artifact",
        f.artifact,
        "--output",
        output,
      ],
      {
        env: {
          ...f.env,
          ATLAS_TEST_INTERRUPT: "1",
          ATLAS_TEST_CHILD_PID_FILE: pidFile,
        },
        encoding: "utf8",
        timeout: 15000,
      },
    );
    assert.equal(result.status, 130, result.stdout + result.stderr);
    const report = JSON.parse(readFileSync(resolve(output, "result.json")));
    assert.equal(report.status, "interrupted");
    assert.equal(report.stages.at(-1).status, "interrupted");
    const pid = Number(readFileSync(pidFile, "utf8"));
    assert.throws(() => process.kill(pid, 0), { code: "ESRCH" });
    assert.equal(existsSync(resolve(output, "checkout")), false);
  } finally {
    rmSync(f.directory, { recursive: true, force: true });
  }
});

function prepareNormalCommand(f) {
  for (const name of [
    "validate-browser.mjs",
    "development/preflight.mjs",
    "validation/inputs.mjs",
    "validation/run-check.mjs",
    "validation/run-process.mjs",
  ]) {
    const destination = resolve(f.root, "scripts", name);
    mkdirSync(resolve(destination, ".."), { recursive: true });
    copyFileSync(new URL(name, import.meta.url), destination);
  }
  writeFileSync(
    resolve(f.root, "web/atlas-ui/package.json"),
    JSON.stringify({
      type: "module",
      engines: { node: "^22.18.0 || ^24.0.0" },
    }),
  );
  writeFileSync(
    resolve(f.root, "rust-toolchain.toml"),
    '[toolchain]\nchannel="1.95.0"\n',
  );
  writeFileSync(
    resolve(f.root, ".gitignore"),
    "bin/\nnode_modules/\ntarget/\n.atlas/\nscratch/\nweb/atlas-ui/dist/\n",
  );
  const program = (name, body) => {
    const file = resolve(f.root, "bin", name);
    writeFileSync(file, "#!/bin/sh\n" + body + "\n");
    chmodSync(file, 0o755);
  };
  program("rustc", "echo 'rustc 1.95.0 (fixture)'");
  program(
    "cargo",
    `case "$1" in
metadata) printf '{"target_directory":"%s/target","packages":[]}\\n' "$PWD";;
--version) echo cargo-fixture;;
build) mkdir -p target/debug/examples; printf '#!/bin/sh\\nexit 0\\n' > target/debug/examples/browser_fixture; chmod +x target/debug/examples/browser_fixture;;
*) exit 91;; esac`,
  );
  program("strip", "exit 0");
  const dist = resolve(f.root, "web/atlas-ui/dist");
  mkdirSync(dist);
  const contract = readAtlasApiContract(f.root).identifier;
  const outputs = {
    "index.html": "candidate A frontend",
    "api-contract.txt": contract + "\n",
  };
  for (const [name, bytes] of Object.entries(outputs))
    writeFileSync(resolve(dist, name), bytes);
  writeFileSync(
    resolve(dist, "build-provenance.json"),
    JSON.stringify({
      version: 1,
      input_sha256: frontendInputIdentity(f.root),
      api_contract: contract,
      sha256: Object.fromEntries(
        Object.entries(outputs).map(([name, bytes]) => [
          name,
          createHash("sha256").update(bytes).digest("hex"),
        ]),
      ),
    }),
  );
  f.git("add", ".");
  f.git("commit", "-qm", "test: normal browser composition");
  f.commit = f.git("rev-parse", "HEAD");
}

for (const fail of [false, true])
  test(`normal browser command composes verified UI, packaging and isolation${fail ? " on failure" : ""}`, () => {
    const f = fixture();
    try {
      prepareNormalCommand(f);
      const output = resolve(f.directory, "normal");
      const result = spawnSync(
        process.execPath,
        [
          resolve(f.root, "scripts/validate-browser.mjs"),
          "--offline",
          "--output",
          output,
          "--",
          "--project=phone-webkit",
        ],
        {
          cwd: f.root,
          env: { ...f.env, ...(fail ? { ATLAS_TEST_FAIL: "1" } : {}) },
          encoding: "utf8",
          timeout: 15000,
        },
      );
      assert.equal(result.status, fail ? 1 : 0, result.stdout + result.stderr);
      assert.match(result.stderr, /Reusing verified frontend bundle/);
      const report = JSON.parse(readFileSync(resolve(output, "result.json")));
      assert.equal(report.commit, f.commit);
      assert.equal(report.status, fail ? "failed" : "passed");
      assert.equal(existsSync(resolve(output, "checkout")), false);
      assert.deepEqual(
        readdirSync(resolve(f.root, "scratch/browser-packages")),
        [],
      );
      assert.equal(
        existsSync(resolve(f.root, "web/atlas-ui/node_modules")),
        false,
      );
    } finally {
      rmSync(f.directory, { recursive: true, force: true });
    }
  });

test("normal browser command rejects dirty source and replay overrides before building", () => {
  const f = fixture();
  try {
    prepareNormalCommand(f);
    const script = resolve(f.root, "scripts/validate-browser.mjs");
    for (const args of [
      ["--artifact", f.artifact],
      ["--commit", "HEAD"],
    ]) {
      const result = spawnSync(process.execPath, [script, ...args], {
        cwd: f.root,
        env: f.env,
        encoding: "utf8",
      });
      assert.notEqual(result.status, 0);
      assert.match(result.stderr, /replay/);
    }
    writeFileSync(resolve(f.root, "uncommitted"), "dirty");
    const result = spawnSync(process.execPath, [script], {
      cwd: f.root,
      env: f.env,
      encoding: "utf8",
    });
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /Commit source changes/);
    assert.equal(existsSync(resolve(f.root, "target")), false);
  } finally {
    rmSync(f.directory, { recursive: true, force: true });
  }
});
