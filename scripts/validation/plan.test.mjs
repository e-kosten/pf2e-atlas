import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  readFileSync,
  writeFileSync,
  existsSync,
  mkdirSync,
  chmodSync,
} from "node:fs";
import { resolve } from "node:path";
import { validationPlan, flatChecks } from "./plan.mjs";
import { fixture } from "./test-fixtures.mjs";

const runner = new URL("./run-check.mjs", import.meta.url).href;
const status = new URL("./status.mjs", import.meta.url).href;
function execute(f, script) {
  return spawnSync(
    process.execPath,
    [
      "--input-type=module",
      "-e",
      `const {runChecks}=await import(${JSON.stringify(runner)});const {inspectEvidence}=await import(${JSON.stringify(status)});${script}`,
    ],
    { cwd: f.cwd, env: f.env, encoding: "utf8", timeout: 8000 },
  );
}

test("mixed application checks share one bundle before the fixed frontend/Rust pair", () => {
  const plan = validationPlan(new Set(["rust", "web", "browser"]));
  assert.deepEqual(plan[2].slice(1), [
    "npm",
    "--prefix",
    "web/atlas-ui",
    "run",
    "verify:bundle",
  ]);
  assert.equal(plan.filter((task) => task.parallel).length, 1);
  assert.deepEqual(plan[3].parallel.frontend[0].slice(1), [
    "npm",
    "--prefix",
    "web/atlas-ui",
    "run",
    "test",
  ]);
  assert(
    flatChecks(plan).some(
      (task) =>
        task.join(" ") === "workspace tests cargo nextest run --workspace",
    ),
  );
  assert(
    !validationPlan(new Set(["rust", "web"]), { parallel: false }).some(
      (task) => task.parallel,
    ),
  );
});

test("tooling-only plan includes actual regressions and excludes application builds", () => {
  const plan = validationPlan(new Set(["tooling"]));
  assert(
    plan.some(
      (task) =>
        task[1] === "node" &&
        task.includes("scripts/validation/run-check.test.mjs"),
    ),
  );
  assert(
    plan.some((task) => task.includes("scripts/git-hooks/test-common.sh")),
  );
  assert(!plan.some((task) => ["npm", "cargo"].includes(task[1])));
});

test("parallel lanes overlap and share one candidate/evidence owner", (t) => {
  const f = fixture(t);
  const first = resolve(f.cwd, "../first");
  const second = resolve(f.cwd, "../second");
  const program = `const fs=require('node:fs');fs.writeFileSync(process.argv[1],'ready');let n=0;const t=setInterval(()=>{if(fs.existsSync(process.argv[2])){clearInterval(t);process.exit(0);}if(++n>100)process.exit(99);},10)`;
  const result = execute(
    f,
    `process.exitCode=await runChecks([{parallel:{frontend:[["frontend",process.execPath,"-e",${JSON.stringify(program)},${JSON.stringify(first)},${JSON.stringify(second)}]],rust:[["rust",process.execPath,"-e",${JSON.stringify(program)},${JSON.stringify(second)},${JSON.stringify(first)}]]}}]);`,
  );
  assert.equal(result.status, 0, result.stderr);
  const records = f.records();
  assert.equal(records.length, 2);
  assert.equal(new Set(records.map((r) => r.dir)).size, 1);
  assert(records.every((r) => r.status === "passed"));
});

for (const failing of ["frontend", "rust"])
  test(`${failing} failure keeps exit 23 and reaps the other lane`, (t) => {
    const f = fixture(t);
    const pid = resolve(f.cwd, "../peer.pid");
    const peer = [
      "peer",
      "sh",
      "-c",
      `trap '' TERM; echo $$ > '${pid}'; sleep 30 & wait`,
    ];
    const fail = ["origin", "sh", "-c", "sleep .2; exit 23"];
    const lanes =
      failing === "frontend"
        ? { frontend: [fail], rust: [peer] }
        : { frontend: [peer], rust: [fail] };
    const result = execute(
      f,
      `process.exitCode=await runChecks([{parallel:${JSON.stringify(lanes)}}]);`,
    );
    assert.equal(result.status, 23, result.stderr);
    const records = f.records();
    assert.equal(records.find((r) => r.label === "origin").status, "failed");
    assert.equal(records.find((r) => r.label === "origin").exit, "23");
    assert.equal(records.find((r) => r.label === "peer").status, "interrupted");
    assert.throws(
      () => process.kill(Number(readFileSync(pid, "utf8")), 0),
      /ESRCH/,
    );
  });

test("receipt inspection is read-only, content-based and surfaces the latest failure", (t) => {
  const f = fixture(t);
  const state = resolve(f.cwd, "../state");
  f.env.STATE_FILE = state;
  const result = execute(
    f,
    `
    const fs=await import('node:fs');const cp=await import('node:child_process');
    const task=['receipt','sh','-c','test ! -f "$STATE_FILE"'];
    const plan=[task];
    const before=inspectEvidence(plan);console.log('BEFORE:'+JSON.stringify(before));
    await runChecks(plan);
    console.log('PASS:'+JSON.stringify(inspectEvidence(plan)));
    cp.execFileSync('git',['commit','--allow-empty','-qm','test: same bytes']);
    console.log('COMMIT:'+JSON.stringify(inspectEvidence(plan)));
    fs.writeFileSync(process.env.STATE_FILE,'fail');
    await runChecks(plan);
    console.log('FAIL:'+JSON.stringify(inspectEvidence(plan)));
  `,
  );
  assert.equal(result.status, 0, result.stderr);
  const report = (prefix) =>
    JSON.parse(
      result.stdout
        .split("\n")
        .find((line) => line.startsWith(prefix))
        .slice(prefix.length),
    );
  assert.equal(report("BEFORE:")[0].state, "missing evidence");
  assert.equal(report("PASS:")[0].state, "matching evidence");
  assert.equal(report("COMMIT:")[0].state, "matching evidence");
  assert.equal(report("FAIL:")[0].state, "invalidated");
  assert.equal(f.records().length, 2);
});

test("receipt inspection explains environment and source changes without exposing secrets", (t) => {
  const f = fixture(t);
  const result = execute(
    f,
    `const fs=await import('node:fs');const plan=[['receipt','sh','-c','true']];await runChecks(plan);process.env.PRIVATE_FIXTURE_SECRET='do-not-print-this';console.log('ENV:'+JSON.stringify(inspectEvidence(plan)));delete process.env.PRIVATE_FIXTURE_SECRET;fs.writeFileSync('code.rs','changed');console.log('SOURCE:'+JSON.stringify(inspectEvidence(plan)));`,
  );
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /PRIVATE_FIXTURE_SECRET/);
  assert.doesNotMatch(result.stdout, /do-not-print-this/);
  assert.match(result.stdout, /source inputs changed/);
});

test("receipt status ignores launch identifiers but keeps compiler environment changes", (t) => {
  const f = fixture(t);
  const result = execute(
    f,
    `
    const task=['receipt','sh','-c','true'];await runChecks([task]);
    process.env.CODEX_TOOL_CALL_ID='new invocation';process.env.XPC_SERVICE_NAME='new launch';
    console.log('META:'+JSON.stringify(inspectEvidence([task])));
    process.env.LIBRARY_PATH='private-compiler-input-value';
    console.log('BUILD:'+JSON.stringify(inspectEvidence([task])));
  `,
  );
  assert.equal(result.status, 0, result.stderr);
  const reports = result.stdout.split("\n");
  assert.match(
    reports.find((line) => line.startsWith("META:")),
    /matching evidence/,
  );
  assert.match(
    reports.find((line) => line.startsWith("BUILD:")),
    /invalidated/,
  );
  assert.match(result.stdout, /LIBRARY_PATH/);
  assert.doesNotMatch(result.stdout, /private-compiler-input-value/);
});

for (const kind of ["npm", "cargo"])
  test(`${kind} input artifacts changing during a successful check invalidate its evidence`, (t) => {
    const f = fixture(t);
    const artifact =
      kind === "npm"
        ? "web/atlas-ui/node_modules/.package-lock.json"
        : "web/atlas-ui/dist/index.html";
    mkdirSync(resolve(f.cwd, artifact, ".."), { recursive: true });
    writeFileSync(resolve(f.cwd, artifact), "before");
    mkdirSync(resolve(f.cwd, "bin"));
    const binary = resolve(f.cwd, "bin", kind);
    writeFileSync(
      binary,
      `#!/bin/sh\nif [ "$1" = --version ]; then echo fixture; exit 0; fi\nprintf after > '${artifact}'\n`,
    );
    chmodSync(binary, 0o755);
    // Ignore fixture executables so artifact mutation is the only changed input.
    writeFileSync(resolve(f.cwd, ".git/info/exclude"), "bin/\n");
    f.env.PATH = resolve(f.cwd, "bin") + ":" + f.env.PATH;
    const args =
      kind === "npm"
        ? ["--prefix", "web/atlas-ui", "run", "test"]
        : ["test", "--workspace"];
    const result = execute(
      f,
      `const task=['artifact',${JSON.stringify(kind)},...${JSON.stringify(args)}];process.exitCode=await runChecks([task]);console.log(JSON.stringify(inspectEvidence([task])));`,
    );
    assert.equal(result.status, 78, result.stderr);
    const record = f.records()[0];
    assert.equal(record.status, "stale");
    assert.notEqual(record.artifacts_sha256, record.end_artifacts_sha256);
    assert.match(result.stdout, /invalidated/);
  });

test("a cancelled peer's receipt error preserves the originating failure and awaits cleanup", (t) => {
  const f = fixture(t);
  const result = execute(
    f,
    `
    const fs=await import('node:fs');const path=await import('node:path');
    const peer=['peer','sh','-c','trap "" TERM; sleep 30 & wait'];
    const origin=['origin',process.execPath,'-e',${JSON.stringify(`const fs=require('node:fs'),path=require('node:path');setTimeout(()=>{const root=process.env.ATLAS_VALIDATION_OUTPUT_ROOT;for(const run of fs.readdirSync(root))for(const check of fs.readdirSync(path.join(root,run))){const dir=path.join(root,run,check);if(check.startsWith('check.')&&fs.readFileSync(path.join(dir,'command.args'),'utf8').startsWith('sh\\n'))fs.mkdirSync(path.join(dir,'result.tsv'));}process.exit(23);},200);`)}];
    process.exitCode=await runChecks([{parallel:{frontend:[origin],rust:[peer]}}]);
  `,
  );
  assert.equal(result.status, 23, result.stderr);
  assert.match(result.stderr, /Peer cleanup\/evidence error/);
});

test("exported command modules import safely from Node stdin", () => {
  const modules = [
    "./run-check.mjs",
    "./tooling.mjs",
    "../development/preflight.mjs",
    "../validate-browser.mjs",
    "../validate-browser-candidate.mjs",
  ].map((name) => new URL(name, import.meta.url).href);
  const result = spawnSync(process.execPath, ["--input-type=module", "-"], {
    input: `for (const name of ${JSON.stringify(modules)}) await import(name);`,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "");
  assert.equal(result.stderr, "");
});
