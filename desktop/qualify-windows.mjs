// Native WebView2/NSIS qualification on a disposable Windows runner. CDP is
// enabled only in the child process environment; no product test API is added.
import assert from "node:assert/strict";
import { execFile, spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createServer } from "node:https";
import { appendFileSync, existsSync, mkdirSync, readdirSync, readFileSync, unlinkSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { createServer as createTcpServer } from "node:net";
import { basename, dirname, join, resolve } from "node:path";
import { promisify } from "node:util";
import { answerDialog, qualifyUpdateGuards, reviewNightly } from "./qualify-windows-update.mjs";
import { qualifyWindowsTabs } from "./qualify-windows-tabs.mjs";

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
function payloadHashes() {
  return Object.fromEntries(readdirSync(installDirectory)
    .filter((name) => /^(atlas(-desktop)?\.exe|.+\.dll|THIRD-PARTY-NOTICES\.md)$/i.test(name))
    .map((name) => [name, sha256(join(installDirectory, name))]));
}
// Exercise the installed window's actual menu command independently of the
// shortcut. WebView2 receives keyboard input through its own CDP session.
function invokeMenu(pid, env, label) {
  ps(`Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class AtlasMenu {
  [DllImport("user32.dll")] public static extern IntPtr GetMenu(IntPtr window);
  [DllImport("user32.dll")] public static extern IntPtr GetSubMenu(IntPtr menu, int index);
  [DllImport("user32.dll")] public static extern int GetMenuItemCount(IntPtr menu);
  [DllImport("user32.dll")] public static extern uint GetMenuItemID(IntPtr menu, int index);
  [DllImport("user32.dll", CharSet=CharSet.Unicode, ExactSpelling=true)] public static extern int GetMenuStringW(IntPtr menu, uint item, StringBuilder text, int length, uint flags);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr window, uint message, UIntPtr command, IntPtr parameter);
}
'@
    $window = (Get-Process -Id ${pid}).MainWindowHandle
    $menu = [AtlasMenu]::GetMenu($window)
    for ($i = 0; $i -lt [AtlasMenu]::GetMenuItemCount($menu); $i++) {
      $submenu = [AtlasMenu]::GetSubMenu($menu, $i)
      for ($j = 0; $j -lt [AtlasMenu]::GetMenuItemCount($submenu); $j++) {
        $label = [Text.StringBuilder]::new(256)
        [void][AtlasMenu]::GetMenuStringW($submenu, $j, $label, $label.Capacity, 0x400)
        if (($label.ToString().Replace('&', '') -split [char]9)[0] -eq ${literal(label)}) {
          $command = [AtlasMenu]::GetMenuItemID($submenu, $j)
          if (![AtlasMenu]::PostMessageW($window, 0x0111, [UIntPtr]$command, [IntPtr]::Zero)) { throw 'Menu dispatch failed' }
          return
        }
      }
    }
    throw 'Menu item missing from the installed Atlas window'`, env);
}
// Identify the added workspace HWND independently of translated titles or focus.
function workspaceWindows(pid, env, close) {
  return ps(`Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class AtlasHostWindows {
  [DllImport("user32.dll")] public static extern IntPtr GetTopWindow(IntPtr parent);
  [DllImport("user32.dll")] public static extern IntPtr GetWindow(IntPtr hwnd, uint relation);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern IntPtr GetMenu(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr hwnd, uint msg, UIntPtr wParam, IntPtr lParam);
}
'@
    for ($window = [AtlasHostWindows]::GetTopWindow([IntPtr]::Zero); $window -ne [IntPtr]::Zero;
        $window = [AtlasHostWindows]::GetWindow($window, 2)) {
      $ownerId = [uint32]0
      [void][AtlasHostWindows]::GetWindowThreadProcessId($window, [ref]$ownerId)
      if ($ownerId -eq ${pid} -and [AtlasHostWindows]::IsWindowVisible($window) -and [AtlasHostWindows]::GetMenu($window) -ne [IntPtr]::Zero) {
        if ($window.ToInt64() -eq ${close ?? 0}) {
          if (![AtlasHostWindows]::PostMessageW($window, 0x10, [UIntPtr]::Zero, [IntPtr]::Zero)) { throw 'Close request failed' }
        }
        $window.ToInt64()
      }
    }`, env).trim().split(/\s+/).filter(Boolean).map(Number);
}
function installedProcesses() {
  const result = ps(`@((Get-CimInstance Win32_Process | Where-Object {
    $_.ExecutablePath -and ([IO.Path]::GetDirectoryName($_.ExecutablePath) -eq ${literal(installDirectory)}) -and
      ($_.Name -in @('atlas.exe', 'atlas-desktop.exe'))
  }) | Select-Object ProcessId, ParentProcessId, Name, CommandLine) | ConvertTo-Json -Compress`);
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
  writeFileSync(join(source, "packs/actions/second.json"), JSON.stringify({ _id: "bbbbbbbbbbbbbbbb", name: "Second Windows fixture", type: "action", system: { description: { value: "Second link destination." }, traits: { value: [] } } }));
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
    const prior = mode === "predecessor";
    const sourceInstaller = prior ? plan.priorInstaller : plan.oldInstaller;
    ps(`$p = Start-Process -FilePath ${literal(sourceInstaller)} -ArgumentList ${literal(`/S /D=${installDirectory}`)} -PassThru -Wait; if ($p.ExitCode -ne 0) { throw "Old installer failed: $($p.ExitCode)" }`, env, 90_000);
    if (!prior) {
      assert.equal(sha256(join(installDirectory, "atlas-desktop.exe")), plan.oldHostSha256);
      assert.equal(sha256(join(installDirectory, "atlas.exe")), plan.oldCliSha256);
    }
    if (mode === "normal") {
      const bootstrap = spawn(join(installDirectory, "atlas-desktop.exe"), [], { env, cwd: home, stdio: "ignore" });
      await nativePage((page) => page.getByRole("button", { name: "Set up Atlas", exact: true }).isVisible());
      invokeMenu(bootstrap.pid, env, "Exit");
      await until("bootstrap-only exit", () => bootstrap.exitCode === 0);
      log("bootstrap-only-exit");
    }
    const setup = ["setup", "--offline", "--no-embeddings", "--json", "--source", source];
    cli(setup, env);
    cli(["lists", "create", "Windows handoff saved work", "--slug", "handoff", "--record", "actions:aaaaaaaaaaaaaaaa", "--json"], env);
    const original = join(home, "pf2e-atlas/workspace.sqlite");
    const selectionPath = join(home, "pf2e-atlas/workspace-selection.json");
    writeFileSync(join(root, "active-case.json"), JSON.stringify({ caseRoot, home, mode, installDirectory, cdpPort,
      targetVersion: plan.nightly, targetHostSha256: plan.targetHostSha256, targetCliSha256: plan.targetCliSha256 }));
    if (prior) {
      const before = payloadHashes();
      const selectionHash = existsSync(selectionPath) ? sha256(selectionPath) : null;
      // A predecessor or a late terminal can reach NSIS without the host preflight.
      ps(`$held = [IO.File]::OpenRead(${literal(join(installDirectory, "atlas.exe"))})
        try {
          $p = Start-Process -FilePath ${literal(plan.directInstaller)} -ArgumentList ${literal(`/S /D=${installDirectory}`)} -PassThru -Wait
          if ($p.ExitCode -eq 0) { throw 'The installer accepted a locked sidecar' }
        } finally { $held.Dispose() }`, env, 90_000);
      assert.deepEqual(payloadHashes(), before);
      assert.equal(existsSync(selectionPath) ? sha256(selectionPath) : null, selectionHash);
      assert(!existsSync(join(caseRoot, "postinstall.json")));
      const showList = ["lists", "show", "handoff", "--json"];
      const saved = cli(showList, env);
      assert.equal(saved.list.name, "Windows handoff saved work");
      assert.deepEqual(saved.items.map((item) => item.record_key), ["actions:aaaaaaaaaaaaaaaa"]);
      log("direct-installer-locked-sidecar-refused", { tag: plan.priorTag, sourceSha256: plan.priorHash, files: before });
      ps(`$p = Start-Process -FilePath ${literal(plan.directInstaller)} -ArgumentList ${literal(`/P /D=${installDirectory}`)} -PassThru -Wait; if ($p.ExitCode -ne 0) { throw "Forward installation failed: $($p.ExitCode)" }`, env, 90_000);
      assert.equal(sha256(join(installDirectory, "atlas-desktop.exe")), plan.directHostHash);
      assert.equal(sha256(join(installDirectory, "atlas.exe")), plan.targetCliSha256);
      assert.equal(existsSync(selectionPath) ? sha256(selectionPath) : null, selectionHash);
      const beforeSetup = cli(["desktop-probe"], env);
      assert.equal(windowsPath(beforeSetup.workspace), windowsPath(original));
      assert.equal(beforeSetup.status, "upgrade_required");
      assert.equal(cli(setup, env).ready, true);
      const afterSetup = cli(["desktop-probe"], env);
      assert.equal(windowsPath(afterSetup.workspace), windowsPath(original));
      assert(["browsable", "ready"].includes(afterSetup.status));
      const migrated = cli(showList, env);
      assert.equal(migrated.list.list_key, saved.list.list_key);
      assert.equal(migrated.list.name, saved.list.name);
      assert.deepEqual(migrated.items.map((item) => item.record_key), saved.items.map((item) => item.record_key));
      assert.equal(existsSync(selectionPath) ? sha256(selectionPath) : null, selectionHash);
      log("published-predecessor-upgraded", { tag: plan.priorTag, files: payloadHashes(),
        beforeSetup: beforeSetup.status, afterSetup: afterSetup.status, listKey: migrated.list.list_key });
      return;
    }
    log("launch-old", { mode, caseRoot });
    const child = spawn(join(installDirectory, "atlas-desktop.exe"), [], { env, cwd: home, stdio: ["ignore", "pipe", "pipe"], windowsHide: false });
    captureHostOutput(child.stdout, join(caseRoot, "host.stdout.log"));
    captureHostOutput(child.stderr, join(caseRoot, "host.stderr.log"));
    child.on("error", (error) => log("host-error", { mode, message: String(error) }));
    child.on("exit", (code, signal) => log("host-exit", { mode, code, signal }));
    const page = await product(plan.stable);
    const runningPath = await page.evaluate(() => window.__ATLAS_DESKTOP_SETUP__?.channel?.workspace_path);
    assert.equal(windowsPath(runningPath), windowsPath(original), "The old host did not open the isolated fixture");
    let second;
    if (mode === "normal") {
      const input = page.getByRole("searchbox", { name: "Search records" });
      await input.fill("unsubmitted fixture search");
      await page.keyboard.press("Control+Comma");
      await expect(page.getByRole("heading", { name: "Settings", exact: true })).toBeVisible();
      await page.keyboard.press("Control+Comma");
      await expect(page.getByRole("tab", { name: "Settings", exact: true })).toHaveCount(1);
      log("settings-shortcut-reused-tab", { mode });
      const terminalRow = page.getByRole("link", { name: "Terminal access" });
      await terminalRow.click();
      await expect(page.getByRole("heading", { name: "Run Atlas in PowerShell" })).toBeVisible();
      const command = await page.getByRole("main").locator("code").innerText();
      const cliPath = join(installDirectory, "atlas.exe");
      const quotedPath = /^& '((?:[^'\u2018-\u201b]|(['\u2018-\u201b])\2)*)'$/.exec(command);
      assert.ok(quotedPath, "Expected a single PowerShell-quoted executable path");
      assert.equal(windowsPath(quotedPath[1].replace(/(['\u2018-\u201b])\1/g, "$1")), windowsPath(cliPath));
      assert.equal(ps(command + " --version", env).trim(), run(cliPath, ["--version"], { env }).trim());
      await page.getByRole("button", { name: "Copy command" }).click();
      await expect(page.getByText("Command copied.", { exact: true })).toBeVisible();
      assert.equal(ps("Get-Clipboard -Raw", env).trimEnd(), command);
      await page.getByRole("link", { name: "Back to Settings" }).click();
      await expect(terminalRow).toBeFocused();
      await page.keyboard.press("Control+w");
      await expect(input).toBeFocused();
      await expect(input).toHaveValue("unsubmitted fixture search");

      log("settings-restored-search-draft", { mode });
      // Keep Settings open in both windows, then deliver a record to a window
      // whose only remaining tab is Settings. The original draft stays mounted.
      invokeMenu(child.pid, env, "Settings…");
      await expect(page.getByRole("heading", { name: "Settings", exact: true })).toBeVisible();
      await page.keyboard.press("Control+n");
      second = await nativePage(async (candidate) =>
        candidate !== page && await candidate.getByRole("button", { name: "Settings", exact: true }).isVisible().catch(() => false));
      await second.getByRole("button", { name: "Settings", exact: true }).click();
      const secondSettings = second.getByRole("heading", { name: "Settings", exact: true });
      await expect(secondSettings).toBeVisible();
      const searchTab = second.getByRole("tab", { name: "Search", exact: true });
      await searchTab.locator("..").getByRole("button", { name: "Close tab" }).click();
      await expect(second.getByRole("tablist")).toHaveCount(0);
      await expect(secondSettings).toBeVisible();
      run(join(installDirectory, "atlas-desktop.exe"), ["pf2e-atlas://v1/record/actions%3Aaaaaaaaaaaaaaaaa"], { env, cwd: home });
      await expect(second.getByRole("heading", { name: "Windows updater fixture", exact: true })).toBeVisible();
      await expect(page.getByRole("heading", { name: "Settings", exact: true })).toBeVisible();
      await page.getByRole("tab", { name: "Search", exact: true }).click();
      await expect(input).toHaveValue("unsubmitted fixture search");
      log("settings-per-window-menu-and-record-link", { mode });
    }
    await page.getByRole("button", { name: "Settings", exact: true }).click();
    await page.getByRole("link", { name: /Release channel/ }).click();
    const channel = page.getByRole("main").filter({ has: page.getByRole("heading", { name: "Release channel", exact: true }) });
    const review = () => reviewNightly(channel, expect, mode === "normal");
    const install = await review();
    await page.screenshot({ path: join(caseRoot, "review.png") });
    const processes = installedProcesses();
    assert(processes.some((entry) => entry.Name === "atlas-desktop.exe"));
    assert(processes.some((entry) => entry.Name === "atlas.exe" && entry.CommandLine?.includes("--desktop-launcher")));
    writeFileSync(join(caseRoot, "before-processes.json"), JSON.stringify(processes, null, 2));
    const lateLock = mode === "late-lock";
    const before = lateLock ? payloadHashes() : null;
    const lockRequest = join(root, "request-late-lock");
    if (lateLock) writeFileSync(lockRequest, "");
    if (mode === "normal") {
      await qualifyUpdateGuards({
        second, channel, install, review, expect, pid: child.pid, env, log,
        ps: (code, env, timeout) => promisify(execFile)("pwsh", ["-NoProfile", "-Command", code], { env, timeout, windowsHide: true }),
        terminal: async () => {
          const session = spawn(join(installDirectory, "atlas.exe"), ["web"], { cwd: home, env, windowsHide: true, stdio: ["ignore", "pipe", "pipe"] });
          let output = "";
          session.stdout.on("data", (bytes) => { output += bytes; });
          session.stderr.on("data", (bytes) => { output += bytes; });
          await until("independent terminal lease", () => /Atlas web service listening at/.test(output), 20_000);
          return session;
        },
        verifyOld: () => {
          assert.equal(sha256(join(installDirectory, "atlas-desktop.exe")), plan.oldHostSha256);
          assert.equal(sha256(join(installDirectory, "atlas.exe")), plan.oldCliSha256);
          const selection = existsSync(selectionPath) ? readJson(selectionPath) : { active: "original", pending: null };
          assert.equal(selection.active, "original");
          assert.equal(selection.pending, null);
          assert.match(JSON.stringify(cli(["lists", "show", "handoff", "--json"], env)), /Windows handoff saved work/);
        },
      });
    } else {
      await install.click();
    }
    if (lateLock) {
      const started = join(root, "installer-started");
      const installerPid = await until("installer reached replacement", () => Number(readFileSync(started, "utf8")), 90_000);
      // Keep the exact installed sidecar open until this NSIS instance exits.
      // A running desktop or active old worker would fail this admission.
      assert(!installedProcesses().some((entry) => entry.Name === "atlas-desktop.exe"));
      ps(`$installer = Get-Process -Id ${installerPid}
        $lock = [IO.File]::OpenRead(${literal(join(installDirectory, "atlas.exe"))})
        try {
          [IO.File]::WriteAllText(${literal(join(root, "lock-ready"))}, "ready")
          if (!$installer.WaitForExit(45000)) { throw 'NSIS did not report the locked executable' }
        } finally { $lock.Dispose() }`, env, 55_000);
      unlinkSync(lockRequest);
      assert.deepEqual(payloadHashes(), before);
      assert(!existsSync(join(caseRoot, "postinstall.json")));
      const pending = readJson(selectionPath);
      assert.equal(pending.active, "original");
      assert.equal(pending.pending.target.version, plan.nightly);
      assert(!pending.pending.previous_resumed);
      if (browser?.isConnected()) await browser.close().catch(() => {});
      browser = null;
      const resumed = spawn(join(installDirectory, "atlas-desktop.exe"), [], { env, cwd: home, windowsHide: false, stdio: "ignore" });
      resumed.on("error", (error) => log("resume-error", { message: String(error) }));
      const recovery = await nativePage((candidate) => candidate.getByRole("button", { name: "Resume my previous workspace" }).isVisible().catch(() => false));
      await recovery.getByRole("button", { name: "Resume my previous workspace" }).click();
      const previous = await product(plan.stable);
      assert(readJson(selectionPath).pending.previous_resumed);
      assert.match(JSON.stringify(cli(["lists", "show", "handoff", "--json"], env)), /Windows handoff saved work/);
      await previous.getByRole("button", { name: "Settings", exact: true }).click();
      await previous.getByRole("link", { name: /Release channel/ }).click();
      await previous.getByRole("button", { name: "Retry pending install" }).click();
      log("late-lock-resumed-and-retried");
    }
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
      const pid = installedProcesses().find((entry) => entry.Name === "atlas-desktop.exe").ProcessId;
      await qualifyWindowsTabs({
        exit: () => invokeMenu(pid, env, "Exit"),
        windows: (close) => workspaceWindows(pid, env, close),
        answer: (title, button) => answerDialog(ps, pid, env, title, button),

        page: target, expect, nativePage, evidence: caseRoot,
        listKey: cli(["lists", "show", "handoff", "--json"], env).list.list_key,
        openLink: (url) => run(join(installDirectory, "atlas-desktop.exe"), [url], { env, cwd: home }),
        stopWorker: () => ps(`$workers = @(Get-CimInstance Win32_Process -Filter "Name='atlas.exe'" | Where-Object {
          $_.ExecutablePath -eq ${literal(join(installDirectory, "atlas.exe"))} -and $_.CommandLine -like '*--managed-worker *'
        }); if ($workers.Count -ne 1) { throw 'Expected one disposable Atlas service worker.' }
        Stop-Process -Id $workers[0].ProcessId -Force`, env),
      });
      await until("previous host exit", () => !installedProcesses().some((entry) => entry.Name === "atlas-desktop.exe"));
      const clean = spawn(join(installDirectory, "atlas-desktop.exe"), [], { env, cwd: home, stdio: "ignore" });
      await product(plan.nightly);
      invokeMenu(clean.pid, env, "Exit");
      await until("clean connected exit", () => clean.exitCode === 0);
      log("normal-complete", { selected: selected.workspace, config });
    } else if (lateLock) {
      const target = await product(plan.nightly);
      assert.equal(readJson(selectionPath).active, "original");
      assert.equal(readJson(selectionPath).pending, null);
      assert.match(JSON.stringify(cli(["lists", "show", "handoff", "--json"], env)), /Windows handoff saved work/);
      await target.screenshot({ path: join(caseRoot, "retried.png") });
      log("late-lock-retry-complete");
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
    try {
      writeFileSync(join(caseRoot, "failure-processes.json"), JSON.stringify(installedProcesses(), null, 2));
    } catch (inventoryError) {
      log("process-inventory-failed", { mode, message: String(inventoryError) });
    }
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
  for (const mode of ["predecessor", "normal", "identity", "ledger", "late-lock"]) await runCase(mode);
  writeFileSync(join(root, "result.json"), JSON.stringify({ status: "passed", commit: plan.commit, version: plan.nightly, cases: ["predecessor", "normal", "identity", "ledger", "late-lock"] }, null, 2));
  log("qualification-passed", { commit: plan.commit });
} finally {
  await new Promise((done) => server.close(done));
}
