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
  fireEvent.click(screen.getByRole("button", { name: "Setup and Updates" }));
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
