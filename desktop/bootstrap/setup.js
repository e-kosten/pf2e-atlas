const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const ui = Object.fromEntries(
  [
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
  ].map((id) => [id, document.getElementById(id)]),
);
let busy = false;
let cancelled = false;

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

function failure(error) {
  busy = false;
  ui.activity.hidden = true;
  ui.diagnostic.hidden = false;
  ui.details.textContent = String(error);
  status(
    "Atlas needs your attention",
    "Try again, or check the details for the next step.",
  );
  action(ui.primary, "Check workspace", () => refresh(false));
  action(ui.secondary);
}

async function work(heading, description, command) {
  if (busy) return;
  busy = true;
  ui.diagnostic.hidden = true;
  ui.result.hidden = true;
  action(ui.primary);
  action(ui.secondary);
  status(heading, description);
  ui.phase.textContent = description;
  ui.count.hidden = true;
  ui.progress.removeAttribute("value");
  ui.activity.hidden = false;
  ui.cancel.hidden = command !== "prepare";
  ui.cancel.disabled = true;
  cancelled = false;
  ui.cancel.onclick = () => {
    cancelled = true;
    ui.cancel.disabled = true;
    void invoke("cancel_setup").catch(failure);
  };
  let result;
  try {
    result = await invoke(command);
  } catch (error) {
    failure(error);
    return;
  }
  busy = false;
  ui.activity.hidden = true;
  await refresh(false);
  if (result) {
    ui.result.textContent = result;
    ui.result.hidden = false;
  }
}

const launch = () =>
  work("Opening Atlas", "Connecting to the local Atlas workspace.", "launch");

function show(inspection, auto) {
  const { probe, running, recovery } = inspection;
  ui.diagnostic.hidden = true;
  action(ui.primary);
  action(ui.secondary);
  const setup = (heading, copy) => () => work(heading, copy, "prepare");
  const usable = probe.status === "ready" || probe.status === "browsable";
  if (usable && !running && !recovery && auto) {
    void launch();
    return;
  }
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
            : "Atlas can open your workspace. You can refresh the Pathfinder reference data here.",
      );
      action(ui.primary, recovery ? "Reconnect" : "Open Atlas", launch);
      action(
        ui.secondary,
        browsable ? "Prepare semantic search" : "Update reference data",
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
        "Upgrade your workspace",
        "Atlas makes a backup before updating the workspace. Your saved lists and encounters carry over.",
      );
      action(
        ui.primary,
        "Upgrade workspace",
        setup(
          "Upgrading Atlas",
          "Preparing your workspace and reference data.",
        ),
      );
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
      failure(probe.message);
  }
}

async function refresh(auto) {
  if (busy) return;
  try {
    show(await invoke("inspect"), auto);
  } catch (error) {
    failure(error);
  }
}

async function init() {
  await listen("atlas-progress", ({ payload }) => {
    if (!busy) return;
    if (!cancelled) ui.cancel.disabled = false;
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
  });
  await listen("atlas-error", ({ payload }) => failure(payload));
  await listen("atlas-inspect", () => refresh(true));
  ui["terminal-form"].onsubmit = async (event) => {
    event.preventDefault();
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

void init().catch(failure);
