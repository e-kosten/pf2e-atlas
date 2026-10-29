import { test } from "node:test";
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  readFileSync,
  readdirSync,
  rmSync,
  existsSync,
  copyFileSync,
  chmodSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { setTimeout as delay } from "node:timers/promises";

const helper = fileURLToPath(new URL("./run-check.sh", import.meta.url));
const shell = `set -eu; tmp_dir=$(mktemp -d); trap 'rm -rf "$tmp_dir"' EXIT; . "$ATLAS_CHECK_HELPER"; verbose=\${ATLAS_TEST_VERBOSE:-0}; run_check "$@"`;
function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), "atlas-check-test-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const cwd = join(root, "checkout");
  mkdirSync(cwd);
  const git = (...args) => {
    const result = spawnSync("git", args, { cwd, encoding: "utf8" });
    assert.equal(result.status, 0, result.stderr);
    return result.stdout.trim();
  };
  mkdirSync(join(cwd, "scripts", "validation"), { recursive: true });
  copyFileSync(helper, join(cwd, "scripts", "validation", "run-check.sh"));
  copyFileSync(
    fileURLToPath(new URL("./run-process.mjs", import.meta.url)),
    join(cwd, "scripts", "validation", "run-process.mjs"),
  );
  mkdirSync(join(cwd, "scripts", "git-hooks"), { recursive: true });
  copyFileSync(
    fileURLToPath(new URL("../verify.sh", import.meta.url)),
    join(cwd, "scripts", "verify.sh"),
  );
  chmodSync(join(cwd, "scripts", "verify.sh"), 0o755);
  copyFileSync(
    fileURLToPath(new URL("../verify-changed.sh", import.meta.url)),
    join(cwd, "scripts", "verify-changed.sh"),
  );
  copyFileSync(
    fileURLToPath(new URL("../git-hooks/common.sh", import.meta.url)),
    join(cwd, "scripts", "git-hooks", "common.sh"),
  );
  git("init", "--quiet");
  git("config", "user.name", "Validation fixture");
  git("config", "user.email", "validation@example.invalid");
  git("config", "commit.gpgsign", "false");
  git("config", "core.hooksPath", "/dev/null");
  writeFileSync(join(cwd, "code.rs"), "pub fn value() {}\n");
  writeFileSync(join(cwd, "README.md"), "Fixture docs\n");
  git("add", ".");
  git("commit", "--quiet", "-m", "test: fixture");
  const env = {
    ...process.env,
    ATLAS_CHECK_HELPER: helper,
    ATLAS_VALIDATION_OUTPUT_ROOT: join(root, "evidence"),
    ATLAS_VALIDATION_HEARTBEAT_SECONDS: "1",
  };
  delete env.ATLAS_VALIDATION_RUN_DIR;
  delete env.GITHUB_STEP_SUMMARY;
  const run = (command, extra = {}) =>
    spawnSync(
      "sh",
      ["-c", shell, "fixture", "fixture check", "sh", "-c", command],
      { cwd, env: { ...env, ...extra }, encoding: "utf8", timeout: 8000 },
    );
  const records = () =>
    readdirSync(env.ATLAS_VALIDATION_OUTPUT_ROOT).flatMap((run) => {
      const dir = join(env.ATLAS_VALIDATION_OUTPUT_ROOT, run);
      return readdirSync(dir)
        .filter((name) => name.startsWith("check."))
        .map((name) => {
          const check = join(dir, name);
          const result = Object.fromEntries(
            readFileSync(join(check, "result.tsv"), "utf8")
              .trim()
              .split("\n")
              .map((line) => line.split("\t")),
          );
          return { dir, check, ...result };
        });
    });
  return { cwd, env, run, records };
}

test("quiet success and command failure retain readable logs and exact outcomes", (t) => {
  const f = fixture(t);
  const good = f.run("printf saved-success");
  assert.equal(good.status, 0, good.stderr);
  assert.doesNotMatch(good.stderr, /saved-success/);
  assert.match(good.stderr, /Validation evidence: /);
  const bad = f.run("printf required-diagnostic; exit 23");
  assert.equal(bad.status, 23, bad.stderr);
  assert.match(bad.stderr, /required-diagnostic/);
  const records = f.records();
  assert.deepEqual(records.map((r) => r.status).sort(), ["failed", "passed"]);
  for (const record of records) {
    assert.match(
      readFileSync(record.log, "utf8"),
      /saved-success|required-diagnostic/,
    );
    assert.match(record.duration_seconds, /^\d+$/);
    assert.equal(record.start_inputs, record.end_inputs);
    assert.equal(
      readFileSync(record.command, "utf8"),
      record.status === "passed"
        ? "sh\n-c\nprintf saved-success\n"
        : "sh\n-c\nprintf required-diagnostic; exit 23\n",
    );
    assert.match(
      readFileSync(join(record.dir, "candidate.tsv"), "utf8"),
      /start_commit\t[0-9a-f]{40}/,
    );
  }
});

test("verbose failure streams and retains output without losing its exit status", (t) => {
  const f = fixture(t);
  const result = f.run("printf streamed-diagnostic; exit 19", {
    ATLAS_TEST_VERBOSE: "1",
  });
  assert.equal(result.status, 19, result.stderr);
  assert.match(result.stderr, /streamed-diagnostic/);
  assert.equal(readFileSync(f.records()[0].log, "utf8"), "streamed-diagnostic");
});

test("source edits, new code and deletions mark successful commands stale", async (t) => {
  for (const command of [
    "printf changed >code.rs",
    "printf new >new.rs",
    "rm code.rs",
    "chmod +x code.rs",
  ]) {
    await t.test(command, (t) => {
      const f = fixture(t);
      const result = f.run(command);
      assert.equal(result.status, 78, result.stderr);
      const [record] = f.records();
      assert.equal(record.status, "stale");
      assert.notEqual(record.start_inputs, record.end_inputs);
    });
  }
});

test("committing identical working bytes and editing unrelated docs preserve evidence", (t) => {
  const f = fixture(t);
  writeFileSync(join(f.cwd, "code.rs"), "pub fn updated() {}\n");
  const result = f.run(
    'git add code.rs; git commit --quiet -m "test: commit same working bytes"; printf updated-docs >README.md',
  );
  assert.equal(result.status, 0, result.stderr);
  const [record] = f.records();
  assert.equal(record.status, "passed");
  assert.equal(record.start_inputs, record.end_inputs);
  assert.notEqual(record.start_commit, record.end_commit);
});

test("heartbeat reports progress and exits with the completed check", (t) => {
  const f = fixture(t);
  const result = f.run("sleep 2; printf complete");
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stderr, /Still running fixture check/);
  const [record] = f.records();
  assert.equal(readFileSync(record.log, "utf8"), "complete");
  assert.throws(() => process.kill(Number(record.heartbeat_pid), 0), /ESRCH/);
});

test("termination records interruption and stops its heartbeat", async (t) => {
  const f = fixture(t);
  const child = spawn(
    "sh",
    ["-c", shell, "fixture", "fixture check", "sleep", "30"],
    { cwd: f.cwd, env: f.env, stdio: ["ignore", "pipe", "pipe"] },
  );
  let stderr = "";
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  const exited = new Promise((resolve) =>
    child.on("exit", (code, signal) => resolve({ code, signal })),
  );
  let phase;
  for (let attempt = 0; attempt < 100; attempt++) {
    if (existsSync(f.env.ATLAS_VALIDATION_OUTPUT_ROOT)) {
      for (const run of readdirSync(f.env.ATLAS_VALIDATION_OUTPUT_ROOT)) {
        const dir = join(f.env.ATLAS_VALIDATION_OUTPUT_ROOT, run);
        for (const check of readdirSync(dir).filter((name) =>
          name.startsWith("check."),
        )) {
          const path = join(dir, check, "phase.pid");
          if (existsSync(path))
            phase = Number(readFileSync(path, "utf8").trim());
        }
      }
    }
    if (phase && /Running fixture check/.test(stderr)) break;
    await delay(20);
  }
  assert(phase, stderr);
  // Let the command and heartbeat enter their wait before interrupting the phase.
  await delay(100);
  process.kill(phase, "SIGTERM");
  const outcome = await Promise.race([
    exited,
    delay(3000).then(() => ({ timeout: true })),
  ]);
  assert.deepEqual(outcome, { code: 143, signal: null }, stderr);
  const [record] = f.records();
  assert.equal(record.status, "interrupted");
  assert.equal(record.exit, "143");
  assert.throws(() => process.kill(Number(record.heartbeat_pid), 0), /ESRCH/);
});

test("the explicit runner CLI retains the same evidence", (t) => {
  const f = fixture(t);
  const result = spawnSync(
    "sh",
    [helper, "CLI check", "sh", "-c", "printf CLI-output"],
    { cwd: f.cwd, env: f.env, encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);
  assert.equal(f.records()[0].label, "CLI check");
  assert.equal(readFileSync(f.records()[0].log, "utf8"), "CLI-output");
});

test("source changes between phases cannot combine evidence from different candidates", (t) => {
  const f = fixture(t);
  const script = `set -eu; . "$ATLAS_CHECK_HELPER"; verbose=0; run_check first true; printf changed >code.rs; run_check second sh -c 'printf should-not-run >unexpected.rs'`;
  const result = spawnSync("sh", ["-c", script], {
    cwd: f.cwd,
    env: f.env,
    encoding: "utf8",
  });
  assert.equal(result.status, 78, result.stderr);
  assert.equal(existsSync(join(f.cwd, "unexpected.rs")), false);
  const records = f.records();
  assert.deepEqual(records.map((r) => r.status).sort(), ["passed", "stale"]);
  const stale = records.find((r) => r.status === "stale");
  assert.notEqual(stale.candidate_start_inputs, stale.candidate_end_inputs);
  assert.match(readFileSync(stale.log, "utf8"), /between checks/);
});

test("committing a deletion and an already-present new file keeps actual-input identity", (t) => {
  const f = fixture(t);
  rmSync(join(f.cwd, "code.rs"));
  writeFileSync(join(f.cwd, "new.rs"), "pub fn replacement() {}\n");
  const result = f.run(
    'git add -A; git commit --quiet -m "test: commit replacement bytes"',
  );
  assert.equal(result.status, 0, result.stderr);
  const [record] = f.records();
  assert.equal(record.start_inputs, record.end_inputs);
  assert.notEqual(record.start_commit, record.end_commit);
});

test("nested validation in another checkout gets its own candidate", (t) => {
  const first = fixture(t);
  const second = fixture(t);
  assert.equal(first.run("true").status, 0);
  const inherited = first.records()[0].dir;
  const result = second.run("true", { ATLAS_VALIDATION_RUN_DIR: inherited });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(second.records().length, 1);
  assert.match(
    readFileSync(join(second.records()[0].dir, "candidate.tsv"), "utf8"),
    new RegExp(second.cwd.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")),
  );
});

function alive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch (error) {
    if (error.code === "ESRCH") return false;
    throw error;
  }
}
async function nestedCancellation(
  t,
  { directPhase = false, signal = "SIGTERM", stubborn = false } = {},
) {
  const f = fixture(t);
  const pidFile = join(f.cwd, "..", "owned-pids");
  const nested = join(f.cwd, "..", "nested.sh");
  writeFileSync(
    nested,
    `${stubborn ? "trap '' TERM\n" : ""}printf '%s\\n' "$$" >>"$OWNED_PIDS"\nsh -c 'printf "%s\\n" "$$" >>"$OWNED_PIDS"; sleep 30 & printf "%s\\n" "$!" >>"$OWNED_PIDS"; wait' &\nwait\n`,
  );
  const child = spawn("sh", [helper, "nested check", "sh", nested], {
    cwd: f.cwd,
    env: { ...f.env, OWNED_PIDS: pidFile },
    stdio: ["ignore", "pipe", "pipe"],
  });
  let stderr = "";
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  const closed = new Promise((resolve) =>
    child.on("close", (code, endedSignal) =>
      resolve({ code, signal: endedSignal }),
    ),
  );
  let pids = [];
  let phase;
  t.after(async () => {
    // These are fresh fixture processes created by this test, never a stored
    // application/process receipt. Clean a failing fixture as well as passing it.
    if (pids[0]) {
      try {
        process.kill(-pids[0], "SIGKILL");
      } catch {}
    }
    if (phase && alive(phase)) {
      try {
        process.kill(phase, "SIGTERM");
      } catch {}
    }
    if (child.exitCode === null) child.kill("SIGTERM");
    await Promise.race([closed, delay(2000, undefined, { ref: false })]);
  });
  for (let attempt = 0; attempt < 150; attempt++) {
    if (existsSync(pidFile))
      pids = [
        ...new Set(
          readFileSync(pidFile, "utf8").trim().split("\n").map(Number),
        ),
      ];
    if (existsSync(f.env.ATLAS_VALIDATION_OUTPUT_ROOT)) {
      for (const run of readdirSync(f.env.ATLAS_VALIDATION_OUTPUT_ROOT)) {
        const dir = join(f.env.ATLAS_VALIDATION_OUTPUT_ROOT, run);
        for (const check of readdirSync(dir).filter((name) =>
          name.startsWith("check."),
        )) {
          const path = join(dir, check, "phase.pid");
          if (existsSync(path))
            phase = Number(readFileSync(path, "utf8").trim());
        }
      }
    }
    if (pids.length === 3 && phase) break;
    await delay(20);
  }
  assert.equal(pids.length, 3, stderr);
  assert(pids.every(alive));
  if (directPhase) process.kill(phase, signal);
  else child.kill(signal);
  const outcome = await Promise.race([
    closed,
    delay(4000, { timeout: true }, { ref: false }),
  ]);
  const expected = { SIGHUP: 129, SIGINT: 130, SIGTERM: 143 }[signal];
  assert.deepEqual(outcome, { code: expected, signal: null }, stderr);
  const [record] = f.records();
  assert.equal(record.status, "interrupted");
  assert.equal(record.exit, String(expected));
  assert.throws(() => process.kill(Number(record.heartbeat_pid), 0), /ESRCH/);
  for (let attempt = 0; attempt < 100 && pids.some(alive); attempt++)
    await delay(20);
  assert.deepEqual(
    pids.filter(alive),
    [],
    "owned nested command processes survived validation cancellation",
  );
}

test("public runner PID forwards TERM and reaps nested shell/sleep processes", async (t) =>
  nestedCancellation(t));
test("direct phase TERM reaps nested shell/sleep processes", async (t) =>
  nestedCancellation(t, { directPhase: true }));
test("public runner escalates cleanup for TERM-ignoring descendants", async (t) =>
  nestedCancellation(t, { stubborn: true }));
test("public runner forwards HUP and INT to owned nested commands", async (t) => {
  for (const signal of ["SIGHUP", "SIGINT"])
    await t.test(signal, async (t) => nestedCancellation(t, { signal }));
});

test("run_check restores the caller's signal traps after a completed phase", (t) => {
  const f = fixture(t);
  const marker = join(f.cwd, "..", "trap-marker");
  const script = `set -eu; . "$ATLAS_CHECK_HELPER"; trap 'printf restored >"$TRAP_MARKER"' TERM; run_check first true; kill -TERM $$; test -f "$TRAP_MARKER"`;
  const result = spawnSync("sh", ["-c", script], {
    cwd: f.cwd,
    env: { ...f.env, TRAP_MARKER: marker },
    encoding: "utf8",
    timeout: 5000,
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(readFileSync(marker, "utf8"), "restored");
});

test("successful command exit also cleans an owned background descendant", (t) => {
  const f = fixture(t);
  const pidFile = join(f.cwd, "..", "background-pid");
  const result = f.run('sleep 30 & printf "%s\\n" "$!" >"$OWNED_PID"; exit 0', {
    OWNED_PID: pidFile,
  });
  const pid = Number(readFileSync(pidFile, "utf8").trim());
  t.after(() => {
    if (alive(pid)) process.kill(pid, "SIGKILL");
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(f.records()[0].status, "passed");
  assert.equal(
    alive(pid),
    false,
    "successful validation left an owned background process",
  );
});

const windowsBootstrap = `
const cp = require("node:child_process");
const fs = require("node:fs");
const actualSpawn = cp.spawn;
cp.spawn = (command, args, options) => {
  const child = actualSpawn(command, args, options);
  fs.appendFileSync(process.env.MOCK_PROCESS_LOG, JSON.stringify({ command, args, pid: child.pid }) + "\\n");
  return child;
};
cp.spawnSync = (command, args) => {
  fs.appendFileSync(process.env.MOCK_PROCESS_LOG, JSON.stringify({ command, args }) + "\\n");
  // Model taskkill against this fixture's live Child; no platform-wide scan.
  process.kill(Number(args[1]), "SIGTERM");
  return { status: 0 };
};
require("node:module").syncBuiltinESMExports();
Object.defineProperty(process, "platform", { value: "win32" });
process.argv = [process.execPath, process.env.SUPERVISOR_FILE, ...JSON.parse(process.env.SUPERVISOR_ARGS)];
import(require("node:url").pathToFileURL(process.env.SUPERVISOR_FILE).href);
`;

test("Windows supervisor uses argv-safe Git Bash dispatch and avoids POSIX group signals", (t) => {
  const f = fixture(t);
  const log = join(f.cwd, "..", "windows-process-log");
  const literal = 'literal; $(printf should-not-execute) "quoted"';
  const result = spawnSync(process.execPath, ["-e", windowsBootstrap], {
    cwd: f.cwd,
    env: {
      ...f.env,
      MOCK_PROCESS_LOG: log,
      SUPERVISOR_FILE: fileURLToPath(
        new URL("./run-process.mjs", import.meta.url),
      ),
      SUPERVISOR_ARGS: JSON.stringify([
        process.execPath,
        "-e",
        "process.stdout.write(process.argv[1])",
        literal,
      ]),
    },
    encoding: "utf8",
    timeout: 5000,
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, literal);
  const records = readFileSync(log, "utf8").trim().split("\n").map(JSON.parse);
  assert.equal(records.length, 1);
  assert.equal(records[0].command, "sh");
  assert.deepEqual(records[0].args.slice(0, 3), ["-c", 'exec "$@"', "sh"]);
  assert.equal(records[0].args.at(-1), literal);
});

test("Windows cancellation request targets taskkill at its live Child tree", async (t) => {
  const f = fixture(t);
  const log = join(f.cwd, "..", "windows-process-log");
  const cancel = join(f.cwd, "..", "cancel-request");
  const child = spawn(process.execPath, ["-e", windowsBootstrap], {
    cwd: f.cwd,
    env: {
      ...f.env,
      MOCK_PROCESS_LOG: log,
      SUPERVISOR_FILE: fileURLToPath(
        new URL("./run-process.mjs", import.meta.url),
      ),
      SUPERVISOR_ARGS: JSON.stringify([
        "--cancel-file",
        cancel,
        "--",
        process.execPath,
        "-e",
        'process.stdout.write("ready"); setInterval(() => {}, 1000)',
      ]),
    },
    stdio: ["ignore", "pipe", "pipe"],
  });
  let output = "";
  child.stdout.on("data", (chunk) => {
    output += chunk;
  });
  let stderr = "";
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  const closed = new Promise((resolve) =>
    child.on("close", (code, signal) => resolve({ code, signal })),
  );
  let owned;
  t.after(async () => {
    if (owned && alive(owned)) process.kill(owned, "SIGKILL");
    if (child.exitCode === null) child.kill("SIGTERM");
    await Promise.race([closed, delay(2000, undefined, { ref: false })]);
  });
  for (let attempt = 0; attempt < 150 && output !== "ready"; attempt++)
    await delay(20);
  assert.equal(output, "ready", stderr);
  owned = JSON.parse(readFileSync(log, "utf8").trim().split("\n")[0]).pid;
  writeFileSync(cancel, "143\n");
  const outcome = await Promise.race([
    closed,
    delay(3000, { timeout: true }, { ref: false }),
  ]);
  assert.deepEqual(outcome, { code: 143, signal: null }, stderr);
  const records = readFileSync(log, "utf8").trim().split("\n").map(JSON.parse);
  assert.deepEqual(records[1], {
    command: "taskkill",
    args: ["/PID", String(owned), "/T", "/F"],
  });
  for (let attempt = 0; attempt < 100 && alive(owned); attempt++)
    await delay(20);
  assert.equal(alive(owned), false);
});

test("format-only selector evidence ignores unrelated edits and detects format input changes", async (t) => {
  for (const surface of ["web", "rust"])
    for (const relevant of [false, true]) {
      await t.test(
        `${surface}: ${relevant ? "format input" : "unrelated script"} changes`,
        (t) => {
          const f = fixture(t);
          const bin = join(f.cwd, "..", "bin");
          mkdirSync(bin);
          const program = surface === "web" ? "npm" : "cargo";
          const stub = join(bin, program);
          writeFileSync(
            stub,
            `#!/bin/sh
case "$*" in *--version*) printf fixture-version;; *) printf changed >"$ATLAS_FORMAT_MUTATION";; esac
`,
          );
          chmodSync(stub, 0o755);
          mkdirSync(join(f.cwd, "web", "atlas-ui", "src"), { recursive: true });
          writeFileSync(
            join(f.cwd, "web", "atlas-ui", "src", "example.ts"),
            "export const value = 0;\n",
          );
          const input =
            surface === "web" ? "web/atlas-ui/src/example.ts" : "code.rs";
          const mutation = relevant
            ? input
            : "scripts/validation/unrelated.mjs";
          const paths = join(f.cwd, "..", "format-paths");
          writeFileSync(
            paths,
            surface === "web" ? input : "crates/example/src/lib.rs",
          );
          const result = spawnSync(
            "sh",
            ["scripts/verify-changed.sh", "--paths-file", paths],
            {
              cwd: f.cwd,
              env: {
                ...f.env,
                PATH: `${bin}:${f.env.PATH}`,
                ATLAS_FORMAT_MUTATION: join(f.cwd, mutation),
              },
              encoding: "utf8",
              timeout: 8000,
            },
          );
          assert.equal(result.status, relevant ? 78 : 0, result.stderr);
          const [record] = f.records();
          assert.equal(record.kind, "iteration");
          assert.equal(
            record.input_scope,
            surface === "web" ? "web-format" : "rust-format",
          );
          assert.equal(record.status, relevant ? "stale" : "passed");
          assert.equal(record.candidate_start_inputs, "per-check");
        },
      );
    }
});

test("public full selector TERM waits for its composite Rust phase and nested command cleanup", async (t) => {
  const f = fixture(t);
  const bin = join(f.cwd, "..", "bin");
  const pidFile = join(f.cwd, "..", "full-owned-pids");
  const paths = join(f.cwd, "..", "full-paths");
  mkdirSync(bin);
  writeFileSync(join(bin, "npm"), "#!/bin/sh\nprintf fixture-npm\n");
  writeFileSync(
    join(bin, "cargo"),
    `#!/bin/sh
case "$1" in
 test) printf '%s\n' "$$" >>"$OWNED_PIDS"; sh -c 'printf "%s\n" "$$" >>"$OWNED_PIDS"; sleep 30 & printf "%s\n" "$!" >>"$OWNED_PIDS"; wait' & wait;;
 *) printf fixture-cargo;;
esac
`,
  );
  chmodSync(join(bin, "npm"), 0o755);
  chmodSync(join(bin, "cargo"), 0o755);
  writeFileSync(paths, "crates/example/src/lib.rs");
  const child = spawn(
    "sh",
    ["scripts/verify-changed.sh", "--paths-file", paths, "--full"],
    {
      cwd: f.cwd,
      env: { ...f.env, PATH: `${bin}:${f.env.PATH}`, OWNED_PIDS: pidFile },
      stdio: ["ignore", "pipe", "pipe"],
    },
  );
  let stderr = "";
  child.stderr.on("data", (chunk) => {
    stderr += chunk;
  });
  const closed = new Promise((resolve) =>
    child.on("close", (code, signal) => resolve({ code, signal })),
  );
  let pids = [];
  t.after(async () => {
    if (pids[0]) {
      try {
        process.kill(-pids[0], "SIGKILL");
      } catch {}
    }
    if (child.exitCode === null) child.kill("SIGTERM");
    await Promise.race([closed, delay(2000, undefined, { ref: false })]);
  });
  for (let attempt = 0; attempt < 200; attempt++) {
    if (existsSync(pidFile))
      pids = [
        ...new Set(
          readFileSync(pidFile, "utf8").trim().split("\n").map(Number),
        ),
      ];
    if (pids.length === 3) break;
    await delay(20);
  }
  assert.equal(pids.length, 3, stderr);
  child.kill("SIGTERM");
  const outcome = await Promise.race([
    closed,
    delay(4000, { timeout: true }, { ref: false }),
  ]);
  assert.deepEqual(outcome, { code: 143, signal: null }, stderr);
  const record = f.records().find((r) => r.label === "workspace tests");
  assert.equal(record.status, "interrupted");
  assert.equal(record.exit, "143");
  for (let attempt = 0; attempt < 100 && pids.some(alive); attempt++)
    await delay(20);
  assert.deepEqual(pids.filter(alive), []);
});
