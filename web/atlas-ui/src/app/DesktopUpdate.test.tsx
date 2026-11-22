import { ConfigProvider } from "antd";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import {
  DesktopReleaseChannel,
  DesktopUpdateShortcut,
  DesktopUpdateSection,
} from "./DesktopUpdate";

const desktopWindow = window as Window & {
  __ATLAS_DESKTOP_SETUP__?: object;
  __TAURI__?: object;
};
const invoke = vi.fn();

afterEach(() => {
  delete desktopWindow.__ATLAS_DESKTOP_SETUP__;
  delete desktopWindow.__TAURI__;
  invoke.mockReset();
  vi.restoreAllMocks();
});

function publish(update: object) {
  desktopWindow.__TAURI__ = { core: { invoke } };
  desktopWindow.__ATLAS_DESKTOP_SETUP__ = { token: "current-document", update };
  fireEvent(window, new Event("atlas-desktop-setup"));
}

it("shows the available action only after checking the current offer, then replaces it with progress", async () => {
  const open = vi.fn();
  render(
    <>
      <DesktopUpdateSection />
      <DesktopUpdateShortcut onOpen={open} />
    </>,
  );
  expect(screen.queryByRole("button")).toBeNull();
  let resolveDetails!: (value: object) => void;
  invoke.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        resolveDetails = resolve;
      }),
  );
  publish({ version: "0.2.3", phase: { kind: "available" }, error: null });
  const install = screen.getByRole("button", { name: "Install and restart" });
  expect(install).toBeDisabled();
  fireEvent.click(screen.getByRole("button", { name: "Update available" }));
  expect(open).toHaveBeenCalledWith("updates");
  expect(invoke.mock.calls.map(([, { action }]) => action)).toEqual(["details"]);
  await act(async () => resolveDetails({ kind: "upgrade" }));
  expect(screen.getByText(/will upgrade your workspace/)).toBeVisible();
  invoke.mockRejectedValueOnce("Couldn’t verify the download.");
  fireEvent.click(install);
  expect(await screen.findByText("Couldn’t verify the download.")).toBeVisible();
  expect(invoke).toHaveBeenLastCalledWith("desktop_update_action", {
    token: "current-document",
    version: "0.2.3",
    action: "install",
  });
  publish({
    version: "0.2.3",
    phase: { kind: "downloading", percent: 46 },
    error: null,
  });
  expect(screen.queryByRole("button", { name: /install and restart/i })).toBeNull();
  expect(screen.queryByText(/will upgrade your workspace/)).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Downloading update · 46%" }));
  expect(open).toHaveBeenLastCalledWith("updates");
  publish({ version: "0.2.3", phase: { kind: "installing" }, error: null });
  expect(screen.getByRole("status")).toHaveTextContent("Installing and restarting…");
});

it.each(["initiator", "focus moved", "inactive window", "another window"])(
  "hands focus to the new phase only for the focused initiator (%s)",
  async (scenario) => {
    vi.spyOn(document, "hasFocus").mockReturnValue(scenario !== "inactive window");
    invoke.mockResolvedValueOnce({ kind: "restart" });
    render(
      <>
        <DesktopUpdateSection />
        <button>Release channel</button>
      </>,
    );
    publish({ version: "0.2.3", phase: { kind: "available" } });
    const install = screen.getByRole("button", { name: "Install and restart" });
    await waitFor(() => expect(install).toBeEnabled());
    const elsewhere = screen.getByRole("button", { name: "Release channel" });
    install.focus();
    if (scenario !== "another window") {
      invoke.mockResolvedValueOnce(null);
      await act(async () => fireEvent.click(install));
    }
    if (scenario === "focus moved") elsewhere.focus();
    publish({ version: "0.2.3", phase: { kind: "downloading", percent: 10 } });
    const status = screen.getByRole("status");
    if (scenario === "initiator") expect(status).toHaveFocus();
    else expect(status).not.toHaveFocus();
    if (scenario === "focus moved") expect(elsewhere).toHaveFocus();
    expect(status.textContent).toBe("Downloading update…");
    elsewhere.focus();
    publish({ version: "0.2.3", phase: { kind: "downloading", percent: 71 } });
    expect(status.textContent).toBe("Downloading update…");
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "71");
    publish({ version: "0.2.3", phase: { kind: "installing" } });
    expect(status.textContent).toBe("Installing and restarting…");
    expect(elsewhere).toHaveFocus();
  },
);

it("reopens a channel installation in either pane without issuing a new request", async () => {
  publish({
    version: "0.2.4",
    switching: true,
    phase: { kind: "downloading", percent: null },
  });
  const updatePane = render(<DesktopUpdateSection />);
  expect(screen.getByRole("status")).toHaveTextContent("Downloading update…");
  expect(screen.getByText("Atlas 0.2.4")).toBeVisible();
  expect(screen.queryByRole("button")).toBeNull();
  updatePane.unmount();
  render(<DesktopReleaseChannel />);
  expect(screen.getByRole("status")).toHaveTextContent("Downloading update…");
  publish({ version: "0.2.4", switching: true, phase: { kind: "installing" } });
  expect(screen.getByRole("status")).toHaveTextContent("Installing and restarting…");
  expect(invoke).not.toHaveBeenCalled();
});

it("keeps a resumed installation on its original released version and destination", async () => {
  const open = vi.fn();
  desktopWindow.__TAURI__ = { core: { invoke } };
  desktopWindow.__ATLAS_DESKTOP_SETUP__ = {
    token: "current-document",
    channel: {
      installed: { tag: "release/v0.1.0", version: "0.1.0" },
      active: "original",
      returning: null,
      pending_version: "0.2.4",
      busy: false,
    },
  };
  render(
    <>
      <DesktopReleaseChannel />
      <DesktopUpdateShortcut onOpen={open} />
    </>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Install pending" }));
  expect(open).toHaveBeenCalledWith("channel");
  expect(
    screen.getByText(/Atlas 0.2.4 still has a pending installation/),
  ).toBeVisible();
  expect(screen.queryByText(/Check Nightly compatibility/)).toBeNull();
  invoke.mockRejectedValueOnce({
    kind: "failed",
    message: "Couldn’t reach the update service.",
  });
  fireEvent.click(screen.getByRole("button", { name: "Retry pending install" }));
  expect(await screen.findByText("Couldn’t reach the update service.")).toBeVisible();
  expect(invoke).toHaveBeenCalledWith("desktop_channel_action", {
    token: "current-document",
    action: { kind: "retry_pending" },
  });
});

it("discards a late preflight when the offered version changes", async () => {
  let resolveOld!: (value: object) => void;
  invoke.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        resolveOld = resolve;
      }),
  );
  render(<DesktopUpdateSection />);
  publish({ version: "0.2.3", phase: { kind: "available" } });
  invoke.mockResolvedValueOnce({ kind: "restart" });
  publish({ version: "0.2.4", phase: { kind: "available" } });
  await waitFor(() =>
    expect(screen.getByRole("button", { name: "Install and restart" })).toBeEnabled(),
  );
  await act(async () => resolveOld({ kind: "upgrade" }));
  expect(screen.queryByText(/will upgrade/)).toBeNull();
  expect(screen.queryByRole("alert")).toBeNull();
  expect(screen.getByText("Atlas 0.2.4 is available")).toBeVisible();
});

it("treats rejected preflight as retryable and distinguishes unknown impact", async () => {
  invoke.mockRejectedValueOnce("Reopen Settings.");
  render(<DesktopUpdateSection />);
  publish({ version: "0.2.3", phase: { kind: "available" } });
  expect(await screen.findByText("Reopen Settings.")).toBeVisible();
  expect(screen.getByRole("button", { name: "Install and restart" })).toBeDisabled();
  invoke.mockResolvedValueOnce({ kind: "unknown" });
  fireEvent.click(screen.getByRole("button", { name: "Retry check" }));
  expect(await screen.findByText(/couldn’t determine whether/)).toBeVisible();
  expect(screen.getByRole("button", { name: "Install and restart" })).toBeEnabled();
});

it("offers both ways to wait for Stable after a workspace compatibility refusal", async () => {
  desktopWindow.__TAURI__ = { core: { invoke } };
  desktopWindow.__ATLAS_DESKTOP_SETUP__ = {
    token: "current-document",
    channel: {
      installed: {
        tag: "nightly/20261116092300.gabcdef123456",
        version: "0.1.0-nightly.20261116092300.gabcdef123456",
      },
      active: "separate",
      workspace_path: "/Atlas/separate-workspace.sqlite",
      other_exists: true,
      returning: null,
      ready: false,
      busy: false,
    },
  };
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <DesktopReleaseChannel />
    </ConfigProvider>,
  );
  expect(screen.queryByText(/Choose which data to use/)).toBeNull();
  expect(screen.queryByRole("button", { name: /compatibility/ })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Switch to Stable…" }));
  fireEvent.click(screen.getByRole("button", { name: "Use an earlier copy" }));
  expect(screen.queryByRole("button", { name: /wait for the next Stable/ })).toBeNull();
  invoke.mockRejectedValueOnce({
    kind: "failed",
    message: "Couldn’t retrieve Stable.",
  });
  fireEvent.click(screen.getByRole("button", { name: "Check Stable compatibility" }));
  expect(await screen.findByText("Couldn’t retrieve Stable.")).toBeVisible();
  expect(screen.queryByRole("button", { name: /wait for the next Stable/ })).toBeNull();
  invoke.mockRejectedValueOnce({
    kind: "unavailable",
    message: "This Stable release can’t open this earlier copy of your data yet.",
  });
  fireEvent.click(screen.getByRole("button", { name: "Check Stable compatibility" }));
  await screen.findByText(
    "This Stable release can’t open this earlier copy of your data yet.",
  );
  expect(
    screen.queryByRole("button", { name: "Install Stable and restart" }),
  ).toBeNull();
  expect(
    screen.getByRole("button", {
      name: "Keep checking for Nightly updates while waiting",
    }),
  ).toBeEnabled();

  invoke.mockResolvedValueOnce(null);
  fireEvent.click(
    screen.getByRole("button", {
      name: "Stay on this build and wait for the next Stable release",
    }),
  );
  await waitFor(() =>
    expect(invoke).toHaveBeenLastCalledWith("desktop_channel_action", {
      token: "current-document",
      action: { kind: "wait", workspace: "original", updates: "hold" },
    }),
  );
});

it("explains creating a separate copy and keeps the reviewed destination through confirmation", async () => {
  desktopWindow.__TAURI__ = { core: { invoke } };
  desktopWindow.__ATLAS_DESKTOP_SETUP__ = {
    token: "current-document",
    channel: {
      installed: { tag: "release/v0.1.0", version: "0.1.0" },
      active: "original",
      workspace_path: "/Atlas/workspace.sqlite",
      other_exists: false,
      returning: null,
      ready: false,
      busy: false,
    },
  };
  render(<DesktopReleaseChannel />);
  expect(screen.queryByText(/Choose which data to use/)).toBeNull();
  expect(screen.queryByRole("button", { name: /compatibility/ })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Switch to Nightly…" }));
  expect(invoke).not.toHaveBeenCalled();
  expect(screen.getByRole("button", { name: "Use my current data" })).toHaveFocus();
  expect(screen.getByRole("button", { name: "Use my current data" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  fireEvent.click(screen.getByRole("button", { name: "Use a separate copy" }));
  expect(
    screen.getByText(
      "Creates a copy of your current data. Changes made to either copy stay separate.",
    ),
  ).toBeVisible();
  invoke.mockResolvedValueOnce({
    id: "reviewed-copy",
    version: "0.2.0-nightly",
    workspace: "separate",
    will_seed: true,
    needs_setup: false,
    will_upgrade: false,
  });
  fireEvent.click(screen.getByRole("button", { name: "Check Nightly compatibility" }));
  const install = await screen.findByRole("button", {
    name: "Install Nightly and restart",
  });
  invoke.mockResolvedValueOnce(null);
  fireEvent.click(install);
  await waitFor(() =>
    expect(invoke).toHaveBeenLastCalledWith("desktop_channel_action", {
      token: "current-document",
      action: { kind: "install", token: "reviewed-copy" },
    }),
  );
});

it("reopens the retained other workspace after the separate workspace became the default", async () => {
  desktopWindow.__TAURI__ = { core: { invoke } };
  desktopWindow.__ATLAS_DESKTOP_SETUP__ = {
    token: "current-document",
    channel: {
      installed: { tag: "release/v0.1.0", version: "0.1.0" },
      active: "separate",
      workspace_path: "/Atlas/separate-workspace.sqlite",
      other_exists: true,
      returning: null,
      ready: false,
      busy: false,
    },
  };
  render(<DesktopReleaseChannel />);
  fireEvent.click(screen.getByRole("button", { name: "Switch to Nightly…" }));
  expect(screen.queryByRole("button", { name: "Use a separate copy" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Use the other copy" }));
  expect(
    screen.getByText(
      "Reopens your other copy with its earlier changes. Neither copy is overwritten.",
    ),
  ).toBeVisible();
  invoke.mockRejectedValueOnce("Try again later.");
  fireEvent.click(screen.getByRole("button", { name: "Check Nightly compatibility" }));
  await screen.findByText("Try again later.");
  expect(invoke).toHaveBeenLastCalledWith("desktop_channel_action", {
    token: "current-document",
    action: { kind: "review", workspace: "original", seed_if_missing: true },
  });
});

it.each(["original", "separate"])(
  "explains current and earlier data without assuming Stable provenance when %s is active",
  async (active) => {
    desktopWindow.__TAURI__ = { core: { invoke } };
    desktopWindow.__ATLAS_DESKTOP_SETUP__ = {
      token: "current-document",
      channel: {
        installed: {
          tag: "nightly/20261119092300.gabcdef123456",
          version: "0.1.0-nightly",
        },
        active,
        other_exists: true,
        returning: null,
        ready: false,
        busy: false,
      },
    };
    render(<DesktopReleaseChannel />);
    fireEvent.click(screen.getByRole("button", { name: "Switch to Stable…" }));
    expect(screen.getByRole("button", { name: "Use my current data" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    expect(
      screen.getByText(
        "Keep the data you’ve been using in Nightly, including your latest changes.",
      ),
    ).toBeVisible();
    fireEvent.click(screen.getByRole("button", { name: "Use an earlier copy" }));
    expect(
      screen.getByText(
        "Use an earlier, separate copy of your data. Changes made in Nightly won’t be copied over. Your current Nightly data will remain available.",
      ),
    ).toBeVisible();
    invoke.mockResolvedValueOnce({
      id: "earlier-copy",
      version: "0.2.0",
      workspace: active === "original" ? "separate" : "original",
      will_seed: false,
      needs_setup: false,
      will_upgrade: false,
    });
    fireEvent.click(screen.getByRole("button", { name: "Check Stable compatibility" }));
    expect(
      await screen.findByText(
        "Atlas will install Stable 0.2.0 and use an earlier copy of your data.",
      ),
    ).toBeVisible();
  },
);

it.each([
  ["hold", "Not checking for Nightly updates."],
  ["continue", "Still checking for Nightly updates."],
])(
  "describes the %s waiting preference without update offers",
  async (updates, status) => {
    desktopWindow.__TAURI__ = { core: { invoke } };
    desktopWindow.__ATLAS_DESKTOP_SETUP__ = {
      token: "current-document",
      channel: {
        installed: {
          tag: "nightly/20261119092300.gabcdef123456",
          version: "0.1.0-nightly",
        },
        active: "separate",
        other_exists: true,
        returning: { workspace: "separate", updates },
        ready: false,
        busy: false,
      },
    };
    render(<DesktopReleaseChannel />);
    expect(
      screen.getByText(`Waiting for a compatible Stable release. ${status}`),
    ).toBeVisible();
  },
);

it("requires a deliberate current-data choice when the saved earlier copy is missing", async () => {
  desktopWindow.__TAURI__ = { core: { invoke } };
  desktopWindow.__ATLAS_DESKTOP_SETUP__ = {
    token: "current-document",
    channel: {
      installed: {
        tag: "nightly/20261119092300.gabcdef123456",
        version: "0.1.0-nightly",
      },
      active: "separate",
      other_exists: false,
      returning: { workspace: "original", updates: "hold" },
      ready: false,
      busy: false,
    },
  };
  render(<DesktopReleaseChannel />);
  expect(
    screen.getByText(
      "The earlier copy of your data is missing. Choose your current data to continue.",
    ),
  ).toBeVisible();
  expect(
    screen.getByRole("button", { name: "Check Stable compatibility" }),
  ).toBeDisabled();
  expect(
    screen.getByRole("button", {
      name: "Stay on this build and wait for the next Stable release",
    }),
  ).toBeDisabled();
  expect(screen.getByRole("button", { name: "Cancel return to Stable" })).toBeEnabled();
  fireEvent.click(screen.getByRole("button", { name: "Use my current data" }));
  expect(
    screen.queryByText(
      "The earlier copy of your data is missing. Choose your current data to continue.",
    ),
  ).toBeNull();
  expect(
    screen.getByRole("button", { name: "Check Stable compatibility" }),
  ).toBeEnabled();
  invoke.mockResolvedValueOnce(null);
  fireEvent.click(
    screen.getByRole("button", {
      name: "Stay on this build and wait for the next Stable release",
    }),
  );
  await waitFor(() =>
    expect(invoke).toHaveBeenLastCalledWith("desktop_channel_action", {
      token: "current-document",
      action: { kind: "wait", workspace: "separate", updates: "hold" },
    }),
  );
});

it("ignores a previous page's review after closing and reopening with current data", async () => {
  desktopWindow.__TAURI__ = { core: { invoke } };
  desktopWindow.__ATLAS_DESKTOP_SETUP__ = {
    token: "current-document",
    channel: {
      installed: {
        tag: "nightly/20261119092300.gabcdef123456",
        version: "0.1.0-nightly",
      },
      active: "separate",
      other_exists: true,
      returning: null,
      ready: false,
      busy: false,
    },
  };
  let resolveOld!: (value: object) => void;
  invoke.mockImplementationOnce(
    () =>
      new Promise((resolve) => {
        resolveOld = resolve;
      }),
  );
  const view = render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <DesktopReleaseChannel />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Switch to Stable…" }));
  fireEvent.click(screen.getByRole("button", { name: "Use an earlier copy" }));
  fireEvent.click(screen.getByRole("button", { name: "Check Stable compatibility" }));
  view.unmount();
  render(<DesktopReleaseChannel />);
  await act(async () =>
    resolveOld({
      id: "earlier-copy",
      version: "0.2.0",
      workspace: "original",
      will_seed: false,
      needs_setup: false,
      will_upgrade: false,
    }),
  );
  fireEvent.click(screen.getByRole("button", { name: "Switch to Stable…" }));
  expect(screen.getByRole("button", { name: "Use my current data" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  expect(
    screen.queryByRole("button", { name: "Install Stable and restart" }),
  ).toBeNull();
  invoke.mockResolvedValueOnce({
    id: "current-copy",
    version: "0.2.0",
    workspace: "separate",
    will_seed: false,
    needs_setup: false,
    will_upgrade: false,
  });
  fireEvent.click(screen.getByRole("button", { name: "Check Stable compatibility" }));
  fireEvent.click(
    await screen.findByRole("button", { name: "Install Stable and restart" }),
  );
  await waitFor(() =>
    expect(invoke).toHaveBeenLastCalledWith("desktop_channel_action", {
      token: "current-document",
      action: { kind: "install", token: "current-copy" },
    }),
  );
});
