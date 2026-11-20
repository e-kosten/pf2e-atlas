// Native WebView2/NSIS qualification on a disposable Windows runner. CDP is
// enabled only in the child process environment; no product test API is added.
import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createServer } from "node:https";
import { appendFileSync, existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { createServer as createTcpServer } from "node:net";
import { basename, dirname, join, resolve } from "node:path";

if (process.platform !== "win32") throw new Error("This qualification requires native Windows.");
if (process.env.GITHUB_ACTIONS !== "true") throw new Error("This installer qualification is restricted to a disposable CI runner.");
const { chromium, expect } = createRequire(new URL("../web/atlas-ui/package.json", import.meta.url))("@playwright/test");
const readJson = (path) => JSON.parse(readFileSync(path, "utf8").replace(/^\uFEFF/, ""));
const plan = readJson(resolve(process.argv[2]));
const root = realpathSync(plan.root);
assert.equal(readFileSync(join(root, "qualification-root.txt"), "utf8").replace(/^\uFEFF/, "").trim(), root);
assert.notEqual(plan.oldHostSha256, plan.targetHostSha256);
assert.notEqual(plan.oldCliSha256, plan.targetCliSha256);
const sha256 = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
const windowsPath = (path) => resolve(path.replace(/^\\\\\?\\/, "")).toLowerCase();
const log = (event, data = {}) => {
  const entry = { at: new Date().toISOString(), event, ...data };
  appendFileSync(join(root, "events.jsonl"), JSON.stringify(entry) + "\n");
  process.stdout.write(JSON.stringify(entry) + "\n");
};
function captureHostOutput(stream, path) {
  let remaining = 1_048_576;
  writeFileSync(path, "");
  stream.on("data", (bytes) => {
    if (remaining > 0) {
      const retained = bytes.subarray(0, remaining);
      appendFileSync(path, retained);
      remaining -= retained.length;
      if (remaining === 0) log("host-log-limit", { file: basename(path), bytes: 1_048_576 });
    }
    // Keep draining after the retention limit so logging cannot block the host.
  });
}
function run(command, args, options = {}) {
  const result = spawnSync(command, args, { encoding: "utf8", timeout: 30_000, windowsHide: true, ...options });
  if (result.error || result.status !== 0) {
    throw new Error(`${basename(command)} ${args.join(" ")} failed: ${result.error ?? result.stderr}\n${result.stdout}`);
  }
  return result.stdout;
}
const ps = (script, env, timeout = 30_000) => run("pwsh", ["-NoProfile", "-Command", script], { env, timeout });
const literal = (value) => "'" + value.replaceAll("'", "''") + "'";
const installDirectory = join(root, "installed");
function installedProcesses() {
  const result = ps(`@((Get-CimInstance Win32_Process | Where-Object {
    $_.ExecutablePath -and ([IO.Path]::GetDirectoryName($_.ExecutablePath) -eq ${literal(installDirectory)}) -and
      ($_.Name -in @('atlas.exe', 'atlas-desktop.exe'))
  }) | Select-Object ProcessId, Name, CommandLine) | ConvertTo-Json -Compress`);
  const value = result.trim() ? JSON.parse(result) : [];
  return Array.isArray(value) ? value : [value];
}
function stopInstalled() {
  // Verify executable ownership immediately before stopping each process.
  ps(`Get-CimInstance Win32_Process | Where-Object {
    $_.ExecutablePath -and ([IO.Path]::GetDirectoryName($_.ExecutablePath) -eq ${literal(installDirectory)}) -and
      ($_.Name -in @('atlas.exe', 'atlas-desktop.exe'))
  } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }`);
}
async function until(label, action, timeout = 60_000) {
  const deadline = Date.now() + timeout;
  let error;
  while (Date.now() < deadline) {
    try { const result = await action(); if (result) return result; } catch (failure) { error = failure; }
    await new Promise((done) => setTimeout(done, 250));
  }
  throw new Error(`Timed out waiting for ${label}${error ? `: ${error}` : ""}`);
}
function cli(args, env) {
  return readEnvelope(run(join(installDirectory, "atlas.exe"), args, { env, cwd: env.HOME }));
}
function readEnvelope(text) {
  const value = JSON.parse(text);
  assert.equal(value.status, "ok", JSON.stringify(value));
  return value.data;
}
const descriptor = readJson(plan.installer + ".transition.json");
assert.equal(descriptor.version, plan.nightly);
assert.equal(descriptor.commit, plan.commit);
const manifestPath = `/releases/download/${descriptor.tag}/atlas-desktop-update.json`;
const packagePath = `/releases/download/${descriptor.tag}/${basename(plan.installer)}`;
const manifest = JSON.stringify({
  version: plan.nightly,
  atlasCompatibility: {
    workspaceSchema: descriptor.workspaceSchema,
    foundryCommit: descriptor.foundryCommit,
    foundryVersion: descriptor.foundryVersion,
  },
  platforms: { "windows-x86_64": {
    url: `https://127.0.0.1:${plan.port}${packagePath}`,
    signature: readFileSync(plan.installer + ".sig", "utf8").trim(),
  } },
});
const served = new Map([
  [packagePath, plan.installer],
  [packagePath + ".transition.json", plan.installer + ".transition.json"],
  [packagePath + ".transition.json.sig", plan.installer + ".transition.json.sig"],
]);
let transportEvents = 0;
const transportLog = (event, data) => {
  if (transportEvents++ < 200) log(event, data);
};
const server = createServer({ key: readFileSync(plan.tlsKey), cert: readFileSync(plan.tlsCert) }, (request, response) => {
  response.once("finish", () => transportLog("release-request", {
    method: request.method, path: request.url?.slice(0, 2048), status: response.statusCode,
  }));
  if (request.url === "/releases/latest") {
    response.writeHead(302, { location: `https://127.0.0.1:${plan.port}/releases/tag/release/v${plan.stable}` }).end();
  } else if (request.url === "/releases/download/nightly/latest/nightly-version.txt") {
    response.end(descriptor.tag + "\n");
  } else if (request.url === manifestPath) {
    response.writeHead(200, { "content-type": "application/json" }).end(manifest);
  } else if (served.has(request.url)) {
    response.end(readFileSync(served.get(request.url)));
  } else {
    response.writeHead(404).end();
  }
});
server.requestTimeout = 15_000;
server.headersTimeout = 10_000;
server.setTimeout(15_000, (socket) => socket.destroy());
server.on("tlsClientError", (error, socket) => {
  transportLog("release-tls-error", { code: error.code, message: error.message.slice(0, 2048) });
  socket.destroy();
});
await new Promise((done, reject) => { server.once("error", reject); server.listen(plan.port, "127.0.0.1", done); });
log("release-server-ready", { port: plan.port });

async function unusedPort() {
  const socket = createTcpServer();
  await new Promise((done, reject) => { socket.once("error", reject); socket.listen(0, "127.0.0.1", done); });
  const port = socket.address().port;
  await new Promise((done) => socket.close(done));
  return port;
}
async function runCase(mode) {
  const caseRoot = join(root, mode);
  const home = join(caseRoot, "home");
  const source = join(caseRoot, "source");
  mkdirSync(join(source, "packs/actions"), { recursive: true });
  mkdirSync(join(source, "static/lang"), { recursive: true });
  mkdirSync(home, { recursive: true });
  writeFileSync(join(source, "module.json"), JSON.stringify({ packs: [{ name: "actions", label: "Actions", type: "Item", path: "packs/actions" }] }));
  writeFileSync(join(source, "static/lang/en.json"), "{}");
  writeFileSync(join(source, "packs/actions/handoff.json"), JSON.stringify({ _id: "aaaaaaaaaaaaaaaa", name: "Windows updater fixture", type: "action", system: { description: { value: "Disposable signed Windows updater qualification." }, traits: { value: [] } } }));
  const cdpPort = await unusedPort();
  const env = { ...process.env, HOME: home, USERPROFILE: home, LOCALAPPDATA: home,
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${cdpPort}`,
    WEBVIEW2_USER_DATA_FOLDER: join(caseRoot, "webview2") };
  // Remove developer smoke flags inherited from a caller. The driver uses real UI.
  delete env.ATLAS_PILOT_SMOKE;
  delete env.ATLAS_PILOT_HOME;
  let browser;
  const nativePage = (predicate) => until("native Atlas page", async () => {
    if (!browser?.isConnected()) browser = await chromium.connectOverCDP(`http://127.0.0.1:${cdpPort}`, { timeout: 2_000 });
    for (const context of browser.contexts()) {
      context.setDefaultTimeout(15_000);
      context.setDefaultNavigationTimeout(30_000);
      for (const page of context.pages()) {
        if (await predicate(page)) return page;
      }
    }
  });
  const product = (version) => nativePage((page) => page.evaluate((version) => window.__ATLAS_DESKTOP_SETUP__?.channel?.installed.version === version, version).catch(() => false));
  try {
    stopInstalled();
    ps(`$p = Start-Process -FilePath ${literal(plan.oldInstaller)} -ArgumentList ${literal(`/S /D=${installDirectory}`)} -PassThru -Wait; if ($p.ExitCode -ne 0) { throw "Old installer failed: $($p.ExitCode)" }`, env, 90_000);
    assert.equal(sha256(join(installDirectory, "atlas-desktop.exe")), plan.oldHostSha256);
    assert.equal(sha256(join(installDirectory, "atlas.exe")), plan.oldCliSha256);
    cli(["setup", "--offline", "--no-embeddings", "--json", "--source", source], env);
    cli(["lists", "create", "Windows handoff saved work", "--slug", "handoff", "--record", "actions:aaaaaaaaaaaaaaaa", "--json"], env);
    const original = join(home, "pf2e-atlas/workspace.sqlite");
    const selectionPath = join(home, "pf2e-atlas/workspace-selection.json");
    writeFileSync(join(root, "active-case.json"), JSON.stringify({ caseRoot, home, mode, installDirectory, cdpPort,
      targetVersion: plan.nightly, targetHostSha256: plan.targetHostSha256, targetCliSha256: plan.targetCliSha256 }));
    log("launch-old", { mode, caseRoot });
    const child = spawn(join(installDirectory, "atlas-desktop.exe"), [], { env, cwd: home, stdio: ["ignore", "pipe", "pipe"], windowsHide: false });
    captureHostOutput(child.stdout, join(caseRoot, "host.stdout.log"));
    captureHostOutput(child.stderr, join(caseRoot, "host.stderr.log"));
    child.on("error", (error) => log("host-error", { mode, message: String(error) }));
    child.on("exit", (code, signal) => log("host-exit", { mode, code, signal }));
    const page = await product(plan.stable);
    const runningPath = await page.evaluate(() => window.__ATLAS_DESKTOP_SETUP__?.channel?.workspace_path);
    assert.equal(windowsPath(runningPath), windowsPath(original), "The old host did not open the isolated fixture");
    await page.getByRole("button", { name: "Setup and Updates", exact: true }).click();
    await page.locator(".desktop-channel__preview summary").click();
    await page.getByRole("button", { name: "Try Nightly…", exact: true }).click();
    const dialog = page.getByRole("dialog", { name: "Change release channel", exact: true });
    if (mode === "normal") await dialog.getByRole("button", { name: "Use a separate copy", exact: true }).click();
    await dialog.getByRole("button", { name: "Check Nightly compatibility", exact: true }).click();
    await expect(dialog.getByRole("button", { name: "Install Nightly and restart", exact: true })).toBeEnabled({ timeout: 30_000 });
    await page.screenshot({ path: join(caseRoot, "review.png") });
    const processes = installedProcesses();
    assert(processes.some((entry) => entry.Name === "atlas-desktop.exe"));
    assert(processes.some((entry) => entry.Name === "atlas.exe" && entry.CommandLine?.includes("--desktop-launcher")));
    writeFileSync(join(caseRoot, "before-processes.json"), JSON.stringify(processes, null, 2));
    await dialog.getByRole("button", { name: "Install Nightly and restart", exact: true }).click();
    const hook = await until("NSIS replacement and race hook", () => existsSync(join(caseRoot, "postinstall.json")) && readJson(join(caseRoot, "postinstall.json")), 90_000);
    assert.equal(hook.oldProcesses, 0);
    assert.equal(hook.hostSha256, plan.targetHostSha256);
    assert.equal(hook.cliSha256, plan.targetCliSha256);
    assert.equal(hook.pending.target.version, plan.nightly);
    await until("isolated fixture restart", () => existsSync(join(caseRoot, "restart.json")), 30_000);
    if (mode === "normal") {
      const target = await product(plan.nightly);
      await expect.poll(() => target.evaluate(() => window.__ATLAS_DESKTOP_SETUP__?.channel?.installed.version), { timeout: 30_000 }).toBe(plan.nightly);
      const runningPath = await target.evaluate(() => window.__ATLAS_DESKTOP_SETUP__?.channel?.workspace_path);
      assert.equal(windowsPath(runningPath), windowsPath(join(home, "pf2e-atlas/separate-workspace.sqlite")), "The target did not open the isolated copy");
      const config = readJson(selectionPath);
      assert.equal(config.active, "separate");
      assert.equal(config.pending, null);
      const selected = cli(["desktop-probe"], env);
      assert.equal(windowsPath(selected.workspace), windowsPath(join(home, "pf2e-atlas/separate-workspace.sqlite")));
      assert.match(JSON.stringify(cli(["lists", "show", "handoff", "--json"], env)), /Windows handoff saved work/);
      assert.match(JSON.stringify(cli(["lists", "show", "handoff", "--index", original, "--json"], env)), /Windows handoff saved work/);
      cli(["lists", "create", "Nightly independent change", "--slug", "nightly-only", "--json"], env);
      assert(!JSON.stringify(cli(["lists", "ls", "--index", original, "--json"], env)).includes("nightly-only"), "The separate copy wrote back into the original");
      const key = (path) => run("python", ["-c", "from pathlib import Path; import sqlite3,sys; db=sqlite3.connect(Path(sys.argv[1]).as_uri()+'?mode=ro',uri=True); print(db.execute('select workspace_key from workspace where id=1').fetchone()[0]); db.close()", windowsPath(path)]).trim();
      assert.notEqual(key(original), key(selected.workspace));
      await target.screenshot({ path: join(caseRoot, "target.png") });
      log("normal-complete", { selected: selected.workspace, config });
    } else {
      const recovery = await nativePage((candidate) => candidate.getByRole("heading", { name: "Choose a workspace", exact: true }).isVisible().catch(() => false));
      await expect(recovery.getByRole("button", { name: "Set up Atlas", exact: true })).not.toBeVisible();
      assert.equal(sha256(hook.database), hook.databaseSha256, "Target startup wrote to refused data");
      assert.equal(sha256(selectionPath), hook.selectionSha256, "Target startup committed a refused selection");
      assert.equal(readJson(selectionPath).pending.target.version, plan.nightly);
      assert(!installedProcesses().some((entry) => entry.Name === "atlas.exe" && entry.CommandLine?.includes("--desktop-launcher")), "A service launched for refused data");
      await recovery.screenshot({ path: join(caseRoot, "refused.png") });
      log("race-refused", { mode, databaseSha256: hook.databaseSha256, pending: hook.pending });
    }
  } catch (error) {
    log("case-failed", { mode, message: String(error) });
    for (const context of browser?.contexts() ?? []) {
      for (const [index, page] of context.pages().entries()) {
        await page.screenshot({ path: join(caseRoot, `failure-${index}.png`) }).catch(() => {});
        writeFileSync(join(caseRoot, `failure-${index}.txt`), await page.locator("body").innerText().catch(() => "unavailable"));
      }
    }
    throw error;
  } finally {
    stopInstalled();
    if (browser?.isConnected()) await browser.close().catch(() => {});
  }
}
try {
  for (const mode of ["normal", "identity", "ledger"]) await runCase(mode);
  writeFileSync(join(root, "result.json"), JSON.stringify({ status: "passed", commit: plan.commit, version: plan.nightly, cases: ["normal", "identity", "ledger"] }, null, 2));
  log("qualification-passed", { commit: plan.commit });
} finally {
  await new Promise((done) => server.close(done));
}
