import { ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { SavedWritesPaused } from "../shared/feedback/SavedWritesPaused";
import { DesktopSetupPanel } from "./DesktopSetupPanel";

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

function attach(openRequest?: number) {
  host.__TAURI__ = { core: { invoke } };
  host.__ATLAS_DESKTOP_SETUP__ = {
    token: "current-document",
    openRequest,
    terminalLinking: true,
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

function renderPanel(writesPaused = false) {
  return render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <SavedWritesPaused value={writesPaused}>
        <DesktopSetupPanel />
      </SavedWritesPaused>
    </ConfigProvider>,
  );
}

it("exposes workspace setup only for an attached desktop document", () => {
  renderPanel();
  expect(screen.queryByRole("button", { name: "Setup and Updates" })).toBeNull();
});

it("keeps release-channel settings inside the desktop setup panel", async () => {
  attach();
  host.__ATLAS_DESKTOP_SETUP__ = {
    ...host.__ATLAS_DESKTOP_SETUP__,
    channel: {
      installed: {
        tag: "nightly/20261118023500.gabcdef123456",
        version: "0.1.0-nightly",
      },
      active: "separate",
      workspace_path: "/Atlas/separate-workspace.sqlite",
      other_exists: true,
      returning: null,
      ready: false,
      busy: false,
    },
  };
  renderPanel();
  expect(screen.queryByRole("button", { name: "Change release channel" })).toBeNull();
  const trigger = screen.getByRole("button", { name: "Setup and Updates" });
  fireEvent.click(trigger);
  const panel = await screen.findByRole("dialog", { name: "Setup and Updates" });
  fireEvent.click(
    within(panel).getByRole("button", { name: "Change release channel" }),
  );
  const channel = await screen.findByRole("dialog", { name: "Change release channel" });
  expect(within(channel).getByText("Nightly")).toBeVisible();
  expect(within(channel).getByText("Version 0.1.0-nightly")).toBeVisible();
  expect(within(channel).queryByText(/with the separate workspace/)).toBeNull();
  fireEvent.keyDown(channel, { key: "Escape", code: "Escape", keyCode: 27 });
  await waitFor(() =>
    expect(screen.queryByRole("dialog", { name: "Change release channel" })).toBeNull(),
  );
  expect(screen.getByRole("dialog", { name: "Setup and Updates" })).toBeVisible();
});

it("keeps the setup warning on the trigger until desktop reports completion", () => {
  attach();
  const updateSetup = (status: string, sequence: number) => {
    host.__ATLAS_DESKTOP_SETUP__ = {
      ...host.__ATLAS_DESKTOP_SETUP__,
      setup: { job: 2, sequence, status },
    };
    fireEvent(window, new Event("atlas-desktop-setup"));
  };
  renderPanel();
  const trigger = screen.getByRole("button", { name: "Setup and Updates" });
  trigger.focus();

  updateSetup("available", 6);
  expect(trigger).toHaveAccessibleName("Setup and Updates (setup incomplete)");
  updateSetup("running", 7);
  expect(trigger).toHaveAccessibleName("Setup and Updates (setup in progress)");
  updateSetup("failed", 8);
  expect(trigger).toHaveAccessibleName("Setup and Updates (setup incomplete)");
  updateSetup("idle", 9);
  expect(trigger).toHaveAccessibleName("Setup and Updates");
  expect(trigger).toHaveFocus();
});

it.each([
  { current: 20, percent: "40" },
  { current: 50, percent: "100" },
])(
  "shows $percent% phase progress while setup is running",
  async ({ current, percent }) => {
    attach(7);
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
    const drawer = await screen.findByRole("dialog", { name: "Setup and Updates" });
    expect(within(drawer).getByText("Preparing records for search")).toBeVisible();
    expect(within(drawer).getByText(`${current} of 50 records`)).toBeVisible();
    const bar = within(drawer).getByRole("progressbar", { name: "Setup progress" });
    expect(bar).toHaveAttribute("aria-valuenow", percent);
    expect(bar).not.toHaveClass("ant-progress-status-success");
    expect(within(drawer).getByRole("button", { name: "Cancel setup" })).toBeVisible();
  },
);

it("omits terminal linking when the desktop host uses an installed CLI path", async () => {
  attach(7);
  host.__ATLAS_DESKTOP_SETUP__ = {
    ...host.__ATLAS_DESKTOP_SETUP__,
    terminalLinking: false,
  };
  renderPanel();
  const drawer = await screen.findByRole("dialog", { name: "Setup and Updates" });
  expect(within(drawer).queryByText("Terminal and agent access")).toBeNull();
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
  fireEvent.click(
    screen.getByRole("button", { name: "Setup and Updates (setup incomplete)" }),
  );
  const resume = await screen.findByRole("button", { name: "Resume semantic setup" });
  expect(resume).toBeDisabled();
  fireEvent.click(resume);
  expect(invoke).not.toHaveBeenCalledWith(
    "desktop_setup_action",
    expect.objectContaining({ action: "resume" }),
  );
});

it("opens a retained native menu request and confirms a different bundled pin", async () => {
  attach(7);
  renderPanel();
  const drawer = await screen.findByRole("dialog", { name: "Setup and Updates" });
  expect(invoke).toHaveBeenCalledWith("desktop_setup_action", {
    action: "acknowledge_open",
    job: 2,
    openRequest: 7,
    token: "current-document",
  });
  const apply = () =>
    within(drawer).getByRole("button", { name: "Apply bundled reference data" });
  fireEvent.click(apply());
  expect(within(drawer).getByText(/may be older/)).toBeVisible();
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

it("applies keyword-only settings and links the fixed terminal command", async () => {
  attach();
  renderPanel();
  fireEvent.click(screen.getByRole("button", { name: "Setup and Updates" }));
  const drawer = await screen.findByRole("dialog", { name: "Setup and Updates" });
  expect(
    within(drawer).queryByRole("button", { name: "Apply search models" }),
  ).toBeNull();
  expect(within(drawer).queryByText(/Editing pauses/)).toBeNull();
  const base = within(drawer).getByRole("checkbox", { name: "BGE Base" });
  expect(base).toBeChecked();
  fireEvent.click(base);
  expect(within(drawer).getByText("Atlas default")).toBeVisible();
  expect(within(drawer).getByText(/Editing pauses/)).toBeVisible();
  expect(
    within(drawer).getByText(/Similar records requires a search model/),
  ).toBeVisible();
  fireEvent.click(within(drawer).getByRole("button", { name: "Apply search models" }));
  await waitFor(() =>
    expect(invoke).toHaveBeenLastCalledWith("desktop_setup_action", {
      action: "apply_models",
      job: 2,
      sequence: 5,
      token: "current-document",
      modelSelection: { profiles: [], preferred: null },
    }),
  );
  invoke.mockResolvedValueOnce({
    state: "linked",
    path: "/disposable/.local/bin/atlas",
    version: "0.1.0",
  });
  fireEvent.click(
    within(drawer).getByRole("button", { name: "Install terminal command" }),
  );
  expect(
    await within(drawer).findByText("The terminal command is installed."),
  ).toBeVisible();
  const verification = within(drawer).getByText("command -v atlas");
  expect(verification).not.toBeVisible();
  fireEvent.click(within(drawer).getByText("Details"));
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
  renderPanel();
  fireEvent.click(screen.getByRole("button", { name: "Setup and Updates" }));
  const drawer = await screen.findByRole("dialog", { name: "Setup and Updates" });
  const button = await within(drawer).findByRole("button", {
    name: "Switch to bundled CLI",
  });
  expect(within(drawer).getByText("0.1.0-rc.6")).not.toBeVisible();
  fireEvent.click(button);
  await waitFor(() => expect(button).not.toHaveClass("ant-btn-loading"));
  expect(invoke).toHaveBeenLastCalledWith("desktop_setup_action", {
    action: "switch_terminal",
    job: 2,
    token: "current-document",
  });
  expect(within(drawer).queryByText("command -v atlas")).toBeNull();
  invoke.mockResolvedValueOnce({ ...existing, state: "linked" });
  fireEvent.click(button);
  expect(
    await within(drawer).findByText("The terminal command is installed."),
  ).toBeVisible();
  const verification = within(drawer).getByText("command -v atlas");
  expect(verification).not.toBeVisible();
  fireEvent.click(within(drawer).getByText("Details"));
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
    fireEvent.click(screen.getByRole("button", { name: "Setup and Updates" }));
    const drawer = await screen.findByRole("dialog", { name: "Setup and Updates" });
    expect(within(drawer).queryByRole("checkbox")).toBeNull();
    expect(within(drawer).getByText("Installed")).toBeVisible();
    expect(
      within(drawer).getByText(/Use the Atlas CLI to add or remove/),
    ).toBeVisible();
    expect(
      within(drawer).getByRole("combobox", { name: "Preferred search model" }),
    ).toBeEnabled();
  },
);
