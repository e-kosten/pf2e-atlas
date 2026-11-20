const { invoke: nativeInvoke } = window.__TAURI__.core;
const pending = new Set();

function track(promise) {
  pending.add(promise);
  const settled = () => pending.delete(promise);
  promise.then(settled, settled);
  return promise;
}

function invoke(command, args) {
  return track(nativeInvoke(command, args));
}
const { listen } = window.__TAURI__.event;
const startupTarget = {
  kind: "WebviewWindow",
  label: window.__TAURI__.window.getCurrentWindow().label,
};
const workspaceTemplate = document.getElementById("workspace-template");
const workspaceContent = workspaceTemplate.content.cloneNode(true);
if (window.location.origin === "http://tauri.localhost") {
  workspaceContent.getElementById("terminal").hidden = true;
}
const ui = Object.fromEntries(
  [
    "opening-status",
    "workspace",
    "heading",
    "description",
    "activity",
    "progress",
    "phase",
    "count",
    "primary",
    "secondary",
    "diagnostic",
    "details",
    "result",
    "cancel",
    "bin",
    "terminal-form",
    "terminal-result",
  ].map((id) => [
    id,
    document.getElementById(id) ?? workspaceContent.getElementById(id),
  ]),
);
let busy = false;
let cancelled = false;
let handingOff = false;
let navigating = false;
let inspection = 0;
let generation;
let nextOperation = 0;
let active;
let openingTimer;
const openingStarted = window.__atlasOpeningStarted ?? Date.now();

function showOpening() {
  ui.workspace.remove();
  if (openingTimer !== undefined || !ui["opening-status"].hidden) return;
  openingTimer = scheduleOpeningStatus(ui["opening-status"], openingStarted);
}

function showWorkspace() {
  clearTimeout(openingTimer);
  openingTimer = undefined;
  ui["opening-status"].hidden = true;
  workspaceTemplate.before(ui.workspace);
}

showOpening();

function action(button, title, handler) {
  button.hidden = !title;
  button.disabled = busy;
  button.textContent = title || "";
  button.onclick = handler || null;
}

function status(heading, description) {
  ui.heading.textContent = heading;
  ui.description.textContent = description;
}

function trace(source, error, force = false) {
  if (!window.__atlasStartupTrace) return;
  const entry = {
    source,
    operation: active?.id,
    navigating,
    busy,
    handingOff,
    message: error?.message || String(error),
    ...(typeof error?.code === "string" ? { code: error.code } : {}),
  };
  // Keep a Web Inspector copy if the document loses IPC while navigating.
  console.debug("[atlas-startup]", entry);
  if (force || !handingOff) {
    void invoke("desktop_startup_trace", { entry }).catch(() => {});
  }
}

if (window.__atlasStartupTrace) {
  const warn = console.warn.bind(console);
  let reportedFallback = false;
  console.warn = (message, ...args) => {
    warn(message, ...args);
    if (
      !reportedFallback &&
      typeof message === "string" &&
      message.startsWith("IPC custom protocol failed")
    ) {
      reportedFallback = true;
      trace("tauri-ipc-fallback", args[0]);
    }
  };
}

function failure(error, source) {
  trace(source, error);
  // The native host owns recovery after it starts replacing this document.
  // A late IPC rejection or event must not repaint setup over the handoff.
  if (navigating) return;
  inspection++;
  showWorkspace();
  busy = false;
  handingOff = false;
  active = undefined;
  ui.activity.hidden = true;
  ui.diagnostic.hidden = false;
  ui.details.textContent = error?.message || String(error);
  if (error?.code === "embedding_seed_unavailable") {
    status(
      "Precomputed embeddings are unavailable",
      "Try the download again, or generate embeddings on this computer. A local build can take an hour or more.",
    );
    action(ui.primary, "Retry download", () =>
      work("Preparing Atlas", "Retrying the download.", "prepare"),
    );
    action(ui.secondary, "Build embeddings locally", () =>
      work(
        "Preparing Atlas",
        "Generating missing embeddings on this computer.",
        "prepare",
        {
          buildEmbeddingsLocally: true,
        },
      ),
    );
    return;
  }
  status(
    "Atlas needs your attention",
    "Try again, or check the details for the next step.",
  );
  action(ui.primary, "Check workspace", () => refresh(false));
  action(ui.secondary);
}

async function work(heading, description, command, args) {
  if (busy || handingOff || generation === undefined) return;
  busy = true;
  const job = { id: ++nextOperation, generation, accepted: false, updates: [] };
  active = job;
  inspection++;
  if (command === "launch") showOpening();
  else showWorkspace();
  ui.diagnostic.hidden = true;
  ui.result.hidden = true;
  action(ui.primary);
  action(ui.secondary);
  status(heading, description);
  ui.phase.textContent = "";
  ui.count.hidden = true;
  ui.progress.removeAttribute("value");
  ui.activity.hidden = false;
  ui.cancel.hidden = command !== "prepare";
  ui.cancel.disabled = true;
  cancelled = false;
  ui.cancel.onclick = () => {
    if (handingOff || active !== job || !job.accepted) return;
    cancelled = true;
    ui.cancel.disabled = true;
    void invoke("cancel_setup", { generation, operation: job.id }).catch(
      (error) => trace("invoke:cancel_setup", error),
    );
  };
  try {
    await invoke(command, { ...args, generation, operation: job.id });
  } catch (error) {
    failure(error, `invoke:${command}`);
    return;
  }
  if (active !== job) return;
  job.accepted = true;
  trace("admission-reply-observed", command, true);
  for (const update of job.updates.splice(0)) receiveUpdate(job, update);
}

async function navigate(job, target) {
  if (job.navigating) return;
  job.navigating = true;
  // A matching Retry can already be queued behind this handoff.
  await Promise.resolve();
  if (active !== job) return;
  ui.cancel.hidden = true;
  const hadSetup = ui.workspace.isConnected;
  showOpening();
  if (hadSetup) {
    await Promise.race([
      new Promise((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(resolve)),
      ),
      new Promise((resolve) => setTimeout(resolve, 200)),
    ]);
  }
  while (active === job && pending.size) await Promise.allSettled([...pending]);
  if (active !== job) return;
  navigating = true;
  location.replace(target);
}

function receiveUpdate(job, { type, value }) {
  if (active !== job) return;
  switch (type) {
    case "progress":
      if (!handingOff) showProgress(value);
      break;
    case "navigate":
      handingOff = true;
      void navigate(job, value);
      break;
    case "completed":
      active = undefined;
      busy = false;
      ui.activity.hidden = true;
      void refresh(false).then(() => {
        if (value && !busy) {
          ui.result.textContent = value;
          ui.result.hidden = false;
        }
      });
      break;
    case "failed":
      failure(value, "bootstrap-job");
      break;
    case "retry":
      retryOpening();
      break;
  }
}

function retryOpening() {
  active = undefined;
  navigating = false;
  handingOff = false;
  busy = false;
  void refresh(false);
}

const launch = () =>
  work("Opening Atlas", "Connecting to the local Atlas workspace.", "launch");

function show(inspection, auto) {
  generation = inspection.generation;
  const { probe, running, recovery, opening_error: openingError } = inspection;
  ui.diagnostic.hidden = true;
  action(ui.primary);
  action(ui.secondary);
  const recoverWorkspace = async (previous) => {
    try {
      await invoke("recover_workspace", { previous });
      await refresh(true);
    } catch (error) {
      failure(error, "workspace-selection");
    }
  };
  if (inspection.selection_error) {
    showWorkspace();
    status("Choose a workspace", [inspection.selection_error, inspection.recovery_release].filter(Boolean).join(" "));
    if (inspection.resume_previous_workspace) {
      action(ui.primary, "Resume my previous workspace", () => recoverWorkspace(true));
    } else {
      action(ui.primary, "Open my original workspace", () => recoverWorkspace(false));
    }
    return;
  }
  const setup = (heading, copy) => () => work(heading, copy, "prepare");
  const usable = probe.status === "ready" || probe.status === "browsable";
  if (usable && openingError) {
    failure(openingError, "persisted-retry");
    status(
      "Atlas couldn’t open",
      "Your workspace is ready. Try opening Atlas again.",
    );
    action(ui.primary, "Retry opening Atlas", launch);
    return;
  }
  if (usable && !running && !recovery && auto) {
    void launch();
    return;
  }
  showWorkspace();
  switch (probe.status) {
    case "ready":
    case "browsable": {
      const browsable = probe.status === "browsable";
      status(
        recovery ? "Reconnect to Atlas" : "Your workspace is ready",
        recovery
          ? "The local service stopped. Reconnect to open a fresh window. Existing windows stay open."
          : browsable
            ? "Your reference data is available. Prepare the search model to use semantic search."
            : "Atlas can open your workspace. Use Setup and Updates in Atlas to manage reference data.",
      );
      action(ui.primary, recovery ? "Reconnect" : "Open Atlas", launch);
      action(
        ui.secondary,
        browsable ? "Prepare semantic search" : null,
        setup(
          "Updating Atlas",
          "Preparing reference data and the search model.",
        ),
      );
      break;
    }
    case "needs_setup":
      status(
        "Prepare your workspace",
        "Atlas will download the Pathfinder reference data and a search model, then build your local index.",
      );
      action(
        ui.primary,
        "Set up Atlas",
        setup(
          "Setting up Atlas",
          "Atlas will open when your workspace is ready.",
        ),
      );
      break;
    case "upgrade_required":
      status(
        "Update your workspace for this version of Atlas",
        [
          "This version needs to update your existing workspace before Atlas can open it. Atlas backs up your saved work before upgrading.",
          inspection.recovery_release,
        ].filter(Boolean).join(" "),
      );
      action(
        ui.primary,
        "Update workspace",
        setup(
          "Updating your workspace",
          "Your saved work will be available when Atlas reopens.",
        ),
      );
      if (inspection.recovery_release) action(ui.secondary, "Open my original workspace", () => recoverWorkspace(false));
      break;
    case "recovery_required":
      status(
        "Resume Atlas setup",
        "Atlas found an interrupted workspace operation. Run setup again to continue.",
      );
      action(
        ui.primary,
        "Resume setup",
        setup("Recovering Atlas", "Checking the existing workspace."),
      );
      break;
    default:
      failure(probe.message, `probe:${probe.status}`);
  }
}

async function refresh(auto) {
  if (busy || handingOff) return;
  const request = ++inspection;
  try {
    const result = await invoke("inspect");
    if (request === inspection) show(result, auto);
  } catch (error) {
    if (request === inspection) failure(error, "invoke:inspect");
    else trace("stale-invoke:inspect", error);
  }
}

function showProgress(payload) {
  if (!cancelled && active?.accepted) ui.cancel.disabled = false;
  ui.phase.textContent = payload.message || "Preparing Atlas.";
  const counted = Number.isFinite(payload.current);
  const hasTotal =
    counted && Number.isFinite(payload.total) && payload.total > 0;
  if (hasTotal) {
    ui.progress.max = payload.total;
    ui.progress.value = payload.current;
  } else {
    ui.progress.removeAttribute("value");
  }
  ui.count.hidden = !counted || !payload.unit;
  if (!ui.count.hidden) {
    const current = payload.current.toLocaleString();
    ui.count.textContent = hasTotal
      ? `${current} of ${payload.total.toLocaleString()} ${payload.unit}`
      : `${current} ${payload.unit} completed`;
  }
}

async function init() {
  await track(
    listen(
      "atlas-bootstrap-job",
      ({ payload }) => {
        const job = active;
        if (
          !job ||
          job.generation !== payload.generation ||
          job.id !== payload.operation
        )
          return;
        if (payload.update.type === "navigate") handingOff = true;
        if (!job.accepted) job.updates.push(payload.update);
        else receiveUpdate(job, payload.update);
      },
      { target: startupTarget },
    ),
  );
  await track(
    listen("atlas-error", ({ payload }) => {
      if (active) trace("native:atlas-error", payload);
      else failure(payload, "native:atlas-error");
    }),
  );
  window.addEventListener("atlas-opening-retry", retryOpening);
  await track(listen("atlas-inspect", () => refresh(true)));
  ui["terminal-form"].onsubmit = async (event) => {
    event.preventDefault();
    if (handingOff || navigating) return;
    try {
      const path = await invoke("enable_cli", { directory: ui.bin.value });
      ui["terminal-result"].textContent =
        `Command linked at ${path}. In your terminal, run command -v atlas and compare the result with this path.`;
    } catch (error) {
      ui["terminal-result"].textContent = String(error);
    }
  };
  await refresh(true);
}

void init().catch((error) => failure(error, "bootstrap-init"));
