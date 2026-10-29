import test from "node:test";
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
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
import { fileURLToPath } from "node:url";

test(
  "native Windows Bash cancellation waits for the owned command tree",
  { skip: process.platform !== "win32", timeout: 45000 },
  (t) => {
    const root = mkdtempSync(resolve(tmpdir(), "atlas-native-cancel-"));
    const checkout = resolve(root, "checkout");
    mkdirSync(checkout);
    const evidence = resolve(root, "evidence");
    const pids = resolve(root, "owned-pids");
    t.after(() => {
      if (existsSync(pids))
        for (const pid of readFileSync(pids, "utf8").trim().split("\n")) {
          try {
            process.kill(Number(pid), 0);
            spawnSync("taskkill", ["/PID", pid, "/T", "/F"], {
              stdio: "ignore",
            });
          } catch {}
        }
      rmSync(root, { recursive: true, force: true });
    });
    assert.equal(
      spawnSync("git", ["init", "--quiet"], { cwd: checkout }).status,
      0,
    );
    for (const [key, value] of [
      ["user.name", "Fixture"],
      ["user.email", "fixture@example.invalid"],
      ["commit.gpgsign", "false"],
      ["core.hooksPath", "/dev/null"],
    ])
      assert.equal(
        spawnSync("git", ["config", key, value], { cwd: checkout }).status,
        0,
      );
    writeFileSync(resolve(checkout, "code.rs"), "fixture\n");
    assert.equal(spawnSync("git", ["add", "."], { cwd: checkout }).status, 0);
    assert.equal(
      spawnSync("git", ["commit", "-qm", "test: fixture"], { cwd: checkout })
        .status,
      0,
    );
    const program = resolve(root, "command.cjs");
    writeFileSync(
      program,
      `const fs=require('node:fs');fs.appendFileSync(process.env.OWNED_PIDS,process.pid+'\\n');if(!process.argv.includes('--child'))require('node:child_process').spawn(process.execPath,[__filename,'--child'],{stdio:'ignore'});setInterval(()=>{},1000);`,
    );
    const driver = resolve(root, "driver.sh");
    writeFileSync(
      driver,
      `#!/bin/sh
sh "$CHECK_HELPER" 'native cancellation' "$NODE_EXEC" "$OWNED_PROGRAM" & owned=$!
count=0
while [ ! -f "$OWNED_PIDS" ] || [ "$(wc -l <"$OWNED_PIDS")" -lt 2 ]; do
  count=$((count+1)); [ "$count" -lt 150 ] || exit 97
  sleep .1
done
kill -TERM "$owned"
status=0
wait "$owned" || status=$?
exit "$status"
`,
    );
    const unix = (path) => path.replaceAll("\\", "/");
    const result = spawnSync("sh", [unix(driver)], {
      cwd: checkout,
      env: {
        ...process.env,
        CHECK_HELPER: unix(
          fileURLToPath(new URL("./run-check.sh", import.meta.url)),
        ),
        NODE_EXEC: unix(process.execPath),
        OWNED_PROGRAM: unix(program),
        OWNED_PIDS: unix(pids),
        ATLAS_VALIDATION_OUTPUT_ROOT: unix(evidence),
      },
      encoding: "utf8",
      timeout: 30000,
    });
    assert.equal(result.status, 143, result.stderr);
    for (const value of readFileSync(pids, "utf8").trim().split("\n"))
      assert.throws(() => process.kill(Number(value), 0), /ESRCH/);
    const run = resolve(evidence, readdirSync(evidence)[0]);
    const check = readdirSync(run).find((name) => name.startsWith("check."));
    const report = readFileSync(resolve(run, check, "result.tsv"), "utf8");
    assert.match(report, /status\tinterrupted/);
    assert.match(report, /exit\t143/);
  },
);
