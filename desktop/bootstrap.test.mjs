import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { setImmediate } from "node:timers/promises";
import { runInNewContext } from "node:vm";
import { test } from "node:test";

const requireWeb = createRequire(
  new URL("../web/atlas-ui/package.json", import.meta.url),
);
const { JSDOM } = requireWeb("jsdom");
const script = readFileSync(
  new URL("./bootstrap/setup.js", import.meta.url),
  "utf8",
);
const html = readFileSync(
  new URL("./bootstrap/index.html", import.meta.url),
  "utf8",
).replace(
  "<!-- opening-status -->",
  '<p id="opening-status" hidden>Opening Atlas</p>',
);

const target = "http://127.0.0.1:4732/search";

async function fixture({ trace = true } = {}) {
  const { window } = new JSDOM(html);
  const listeners = new Map();
  const calls = [];
  const navigations = [];
  const post = (update, { generation = 4, operation = 1 } = {}) =>
    listeners.get("atlas-bootstrap-job")({
      payload: { generation, operation, update },
    });
  window.__TAURI__ = {
    event: { listen: async (name, listener) => listeners.set(name, listener) },
    window: { getCurrentWindow: () => ({ label: "atlas-primary" }) },
    core: {
      invoke(command, args) {
        return new Promise((resolve, reject) =>
          calls.push({ command, args, resolve, reject }),
        );
      },
    },
  };
  window.__atlasStartupTrace = trace;
  runInNewContext(script, {
    window,
    document: window.document,
    location: { replace: (value) => navigations.push(value) },
    console: { debug() {}, warn() {} },
    requestAnimationFrame: (resolve) => resolve(),
    setTimeout,
    clearTimeout,
    scheduleOpeningStatus() {},
  });
  await setImmediate();
  function take(command) {
    const index = calls.findIndex((call) => call.command === command);
    assert.notEqual(index, -1, `Missing ${command}`);
    return calls.splice(index, 1)[0];
  }
  return {
    window,
    post,
    listeners,
    take,
    navigations,
    close: () => window.close(),
  };
}

test("armed navigation waits for acceptance, terminal work and diagnostic IPC", async () => {
  const f = await fixture();
  try {
    f.take("inspect").resolve({
      generation: 4,
      probe: { status: "needs_setup" },
    });
    await setImmediate();
    f.window.document
      .getElementById("terminal-form")
      .dispatchEvent(new f.window.Event("submit", { cancelable: true }));
    f.window.document.getElementById("primary").click();
    const terminal = f.take("enable_cli");
    const prepare = f.take("prepare");
    assert.equal(prepare.args.generation, 4);
    assert.equal(prepare.args.operation, 1);
    f.post({ type: "navigate", value: target });
    await setImmediate();
    assert.deepEqual(f.navigations, []);
    prepare.resolve();
    await setImmediate();
    const trace = f.take("desktop_startup_trace");
    assert.equal(trace.args.entry.source, "admission-reply-observed");
    terminal.resolve("/disposable/atlas");
    await setImmediate();
    assert.deepEqual(f.navigations, []);
    // Unrelated startup events must not orphan the scoped operation.
    f.listeners.get("atlas-error")({
      payload: "An unrelated menu action failed.",
    });
    f.post({ type: "navigate", value: target });
    trace.resolve();
    await setImmediate();
    assert.deepEqual(f.navigations, [target]);
  } finally {
    f.close();
  }
});

test("an expired handoff restores Retry and ignores later navigation for the old attempt", async () => {
  const f = await fixture();
  try {
    f.take("inspect").resolve({ generation: 4, probe: { status: "ready" } });
    await setImmediate();
    f.take("launch").resolve();
    await setImmediate();
    const trace = f.take("desktop_startup_trace");
    f.post({ type: "navigate", value: target });
    f.post({ type: "retry" });
    trace.resolve();
    await setImmediate();
    f.take("inspect").resolve({
      generation: 4,
      probe: { status: "ready" },
      opening_error: "The startup page did not open the verified Atlas view.",
    });
    await setImmediate();
    f.post({ type: "navigate", value: target });
    assert.deepEqual(f.navigations, []);
    assert.equal(
      f.window.document.getElementById("primary").textContent,
      "Retry opening Atlas",
    );
  } finally {
    f.close();
  }
});

test("a Retry buffered with the handoff wins when admission settles", async () => {
  const f = await fixture({ trace: false });
  try {
    f.take("inspect").resolve({ generation: 4, probe: { status: "ready" } });
    await setImmediate();
    f.post({ type: "navigate", value: target });
    f.post({ type: "retry" });
    f.take("launch").resolve();
    await setImmediate();
    f.take("inspect").resolve({
      generation: 4,
      probe: { status: "ready" },
      opening_error: "The startup page changed.",
    });
    await setImmediate();
    assert.deepEqual(f.navigations, []);
    assert.equal(
      f.window.document.getElementById("primary").textContent,
      "Retry opening Atlas",
    );
  } finally {
    f.close();
  }
});

test("the opening cover follows the resolved product scheme over the native fallback", () => {
  const { window } = new JSDOM("", { url: target });
  try {
    const { document } = window;
    const prepaint = readFileSync(
      new URL("./src-tauri/src/tab_links.js", import.meta.url),
      "utf8",
    );
    const cover = readFileSync(
      new URL("./bootstrap/opening-handoff.js", import.meta.url),
      "utf8",
    ).replace(
      "/* status markup */ null",
      JSON.stringify("<p hidden>Opening Atlas</p>"),
    );
    window.localStorage.setItem("atlas-ui-color-scheme", "dark");
    window.__atlasOpeningDark = false;
    window.__atlasPrimaryOpening = true;
    runInNewContext(`${prepaint}\n${cover}`, {
      window,
      document,
      location: window.location,
      localStorage: window.localStorage,
      sessionStorage: window.sessionStorage,
      performance: { getEntriesByType: () => [] },
      MutationObserver: window.MutationObserver,
      scheduleOpeningStatus() {},
      clearTimeout,
    });
    assert.equal(
      document.getElementById("atlas-opening-cover").style.colorScheme,
      "dark",
    );
  } finally {
    window.close();
  }
});
