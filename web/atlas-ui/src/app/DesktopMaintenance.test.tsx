import { ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { SavedWritesPaused } from "../shared/feedback/SavedWritesPaused";
import { DesktopSearchData, DesktopTerminal } from "./DesktopMaintenance";
import { DesktopSettingsControls } from "./DesktopSettings";

const host = window as Window & {
  __ATLAS_DESKTOP_SETUP__?: object;
  __TAURI__?: object;
};
const invoke = vi.fn();

afterEach(() => {
  delete host.__ATLAS_DESKTOP_SETUP__;
  delete host.__TAURI__;
  invoke.mockReset();
});

function attach() {
  host.__TAURI__ = { core: { invoke } };
  host.__ATLAS_DESKTOP_SETUP__ = {
    token: "current-document",
    reference: {
      revision: "1111111111111111111111111111111111111111",
      target: "2222222222222222222222222222222222222222",
      targetVersion: "8.6.1",
      managed: true,
      available: true,
      modelsAvailable: true,
      canChangeModels: true,
      models: [
        { id: "bge-base", label: "BGE Base", installed: true, usable: true },
        {
          id: "embeddinggemma",
          label: "EmbeddingGemma",
          installed: false,
          usable: false,
        },
      ],
      preferred: "bge-base",
    },
    setup: { job: 2, sequence: 5, status: "idle" },
  };
  invoke.mockImplementation(async (_command, { action }) =>
    action === "read_terminal"
      ? { state: "missing", path: "/disposable/.local/bin/atlas", version: "0.1.0" }
      : null,
  );
}

function renderPanel(
  writesPaused = false,
  section: "data" | "terminal" | "toolbar" = "data",
) {
  return render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <SavedWritesPaused value={writesPaused}>
        {section === "data" ? (
          <DesktopSearchData />
        ) : section === "terminal" ? (
          <DesktopTerminal />
        ) : (
          <DesktopSettingsControls />
        )}
      </SavedWritesPaused>
    </ConfigProvider>,
  );
}

it("exposes desktop actions only for an attached document", () => {
  renderPanel(false, "toolbar");
  expect(screen.queryByRole("button")).toBeNull();
});

it("opens the data pane from the workspace while setup is active or interrupted", async () => {
  attach();
  const updateSetup = (status: string, sequence: number) => {
    host.__ATLAS_DESKTOP_SETUP__ = {
      ...host.__ATLAS_DESKTOP_SETUP__,
      setup: { job: 2, sequence, status },
    };
    fireEvent(window, new Event("atlas-desktop-setup"));
  };
  renderPanel(false, "toolbar");
  updateSetup("running", 6);
  fireEvent.click(screen.getByRole("button", { name: "Setup in progress" }));
  expect(invoke).toHaveBeenCalledWith("desktop_settings_open", {
    token: "current-document",
    destination: "data",
  });
  updateSetup("available", 7);
  expect(screen.getByRole("button", { name: "Resume setup" })).toBeVisible();
  updateSetup("idle", 8);
  expect(screen.queryByRole("button", { name: "Resume setup" })).toBeNull();
});

it.each([
  { current: 20, percent: "40" },
  { current: 50, percent: "100" },
])(
  "shows $percent% phase progress while setup is running",
  async ({ current, percent }) => {
    attach();
    host.__ATLAS_DESKTOP_SETUP__ = {
      ...host.__ATLAS_DESKTOP_SETUP__,
      setup: {
        job: 2,
        sequence: 5,
        status: "running",
        progress: {
          message: "Preparing records for search",
          current,
          total: 50,
          unit: "records",
        },
      },
    };
    renderPanel();
    expect(screen.getByText("Preparing records for search")).toBeVisible();
    expect(screen.getByText(`${current} of 50 records`)).toBeVisible();
    const bar = screen.getByRole("progressbar", { name: "Setup progress" });
    expect(bar).toHaveAttribute("aria-valuenow", percent);
    expect(bar).not.toHaveClass("ant-progress-status-success");
    expect(screen.getByRole("button", { name: "Cancel setup" })).toBeVisible();
  },
);

it("keeps terminal discovery out of Search and data", () => {
  attach();
  renderPanel();
  expect(invoke).not.toHaveBeenCalledWith(
    "desktop_setup_action",
    expect.objectContaining({ action: "read_terminal" }),
  );
});

it("prevents resuming while another operation holds the workspace write lock", async () => {
  attach();
  host.__ATLAS_DESKTOP_SETUP__ = {
    ...host.__ATLAS_DESKTOP_SETUP__,
    setup: { job: 2, sequence: 5, status: "available", can_resume: true },
  };
  renderPanel(true);
  const resume = await screen.findByRole("button", { name: "Resume semantic setup" });
  expect(resume).toBeDisabled();
  fireEvent.click(resume);
  expect(invoke).not.toHaveBeenCalledWith(
    "desktop_setup_action",
    expect.objectContaining({ action: "resume" }),
  );
});

it("requires confirmation before applying a different bundled pin", async () => {
  attach();
  renderPanel();
  const apply = () =>
    screen.getByRole("button", { name: "Apply bundled reference data" });
  fireEvent.click(apply());
  expect(screen.getByText(/may be older/)).toBeVisible();
  expect(invoke).not.toHaveBeenCalledWith(
    "desktop_setup_action",
    expect.objectContaining({ action: "apply_reference" }),
  );
  fireEvent.click(apply());
  expect(invoke).toHaveBeenLastCalledWith("desktop_setup_action", {
    action: "apply_reference",
    job: 2,
    sequence: 5,
    token: "current-document",
  });
});

it("applies keyword-only settings", async () => {
  attach();
  renderPanel();
  expect(screen.queryByRole("button", { name: "Apply search models" })).toBeNull();
  expect(screen.queryByText(/Editing pauses/)).toBeNull();
  const base = screen.getByRole("checkbox", { name: "BGE Base" });
  expect(base).toBeChecked();
  fireEvent.click(base);
  expect(screen.getByText("Atlas default")).toBeVisible();
  expect(screen.getByText(/Editing pauses/)).toBeVisible();
  expect(screen.getByText(/Similar records requires a search model/)).toBeVisible();
  fireEvent.click(screen.getByRole("button", { name: "Apply search models" }));
  await waitFor(() =>
    expect(invoke).toHaveBeenLastCalledWith("desktop_setup_action", {
      action: "apply_models",
      job: 2,
      sequence: 5,
      token: "current-document",
      modelSelection: { profiles: [], preferred: null },
    }),
  );
});

it("links and verifies the bundled terminal command", async () => {
  attach();
  renderPanel(false, "terminal");
  await screen.findByRole("button", { name: "Install terminal command" });
  invoke.mockResolvedValueOnce({
    state: "linked",
    path: "/disposable/.local/bin/atlas",
    version: "0.1.0",
  });
  fireEvent.click(screen.getByRole("button", { name: "Install terminal command" }));
  expect(await screen.findByText("The terminal command is installed.")).toBeVisible();
  const verification = screen.getByText("command -v atlas");
  expect(verification).not.toBeVisible();
  fireEvent.click(screen.getByText("Command details"));
  expect(verification).toBeVisible();
  expect(invoke).toHaveBeenLastCalledWith("desktop_setup_action", {
    action: "enable_terminal",
    job: 2,
    token: "current-document",
  });
});

it("requests a native switch and keeps the option available after cancellation", async () => {
  attach();
  const existing = {
    state: "replaceable",
    path: "/disposable/.local/bin/atlas",
    version: "0.1.0-rc.6",
  };
  invoke.mockImplementation(async (_command, { action }) =>
    action === "read_terminal" ? existing : null,
  );
  renderPanel(false, "terminal");
  const button = await screen.findByRole("button", {
    name: "Switch to bundled CLI",
  });
  expect(screen.getByText("0.1.0-rc.6")).not.toBeVisible();
  fireEvent.click(button);
  await waitFor(() => expect(button).not.toHaveClass("ant-btn-loading"));
  expect(invoke).toHaveBeenLastCalledWith("desktop_setup_action", {
    action: "switch_terminal",
    job: 2,
    token: "current-document",
  });
  expect(screen.queryByText("command -v atlas")).toBeNull();
  invoke.mockResolvedValueOnce({ ...existing, state: "linked" });
  fireEvent.click(button);
  expect(await screen.findByText("The terminal command is installed.")).toBeVisible();
  const verification = screen.getByText("command -v atlas");
  expect(verification).not.toBeVisible();
  fireEvent.click(screen.getByText("Command details"));
  expect(verification).toBeVisible();
});

it.each([
  { managed: false, available: true },
  { managed: true, available: false },
])(
  "shows CLI guidance and keeps the preference editable without a desktop model action (%j)",
  async (reference) => {
    attach();
    const attachment = host.__ATLAS_DESKTOP_SETUP__ as {
      reference: { managed: boolean; available: boolean; canChangeModels: boolean };
    };
    Object.assign(attachment.reference, reference, { canChangeModels: false });
    renderPanel();
    expect(screen.queryByRole("checkbox")).toBeNull();
    expect(screen.getByText("Installed")).toBeVisible();
    expect(screen.getByText(/Use the Atlas CLI to add or remove/)).toBeVisible();
    expect(
      screen.getByRole("combobox", { name: "Preferred search model" }),
    ).toBeEnabled();
  },
);
