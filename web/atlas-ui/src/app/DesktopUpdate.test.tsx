import { ConfigProvider } from "antd";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import {
  DesktopReleaseChannel,
  DesktopStableUpdate,
  DesktopUpdate,
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
});

it("keeps channel controls out of ordinary browser sessions and the quiet top bar", () => {
  const view = render(
    <>
      <DesktopReleaseChannel />
      <DesktopStableUpdate />
    </>,
  );
  expect(screen.queryByRole("button")).toBeNull();
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
  view.rerender(<DesktopStableUpdate />);
  expect(screen.queryByRole("button")).toBeNull();
});

it("offers an update only when the desktop host reports one and installs only after confirmation", async () => {
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <DesktopUpdate />
    </ConfigProvider>,
  );
  expect(screen.queryByRole("button", { name: "Update" })).toBeNull();

  desktopWindow.__TAURI__ = { core: { invoke } };
  desktopWindow.__ATLAS_DESKTOP_SETUP__ = {
    token: "current-document",
    update: { version: "0.2.3", phase: { kind: "available" }, error: null },
  };
  const warning = "Atlas will be unavailable during workspace setup.";
  invoke.mockResolvedValueOnce(warning);
  fireEvent(window, new Event("atlas-desktop-setup"));
  fireEvent.click(screen.getByRole("button", { name: "Update" }));

  const dialog = await screen.findByRole("dialog", { name: "Update Atlas Desktop" });
  await within(dialog).findByText(warning);
  expect(invoke.mock.calls.map(([, { action }]) => action)).toEqual(["details"]);

  invoke.mockRejectedValueOnce(
    "Atlas couldn’t verify the downloaded update. It won’t be installed.",
  );
  fireEvent.click(within(dialog).getByRole("button", { name: "Install update" }));
  await within(dialog).findByText(
    "Atlas couldn’t verify the downloaded update. It won’t be installed.",
  );
  expect(invoke).toHaveBeenLastCalledWith("desktop_update_action", {
    token: "current-document",
    version: "0.2.3",
    action: "install",
  });
  expect(within(dialog).getByRole("button", { name: "Install update" })).toBeEnabled();

  fireEvent.click(within(dialog).getByRole("button", { name: "Later" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  expect(screen.getByRole("button", { name: "Update" })).toBeEnabled();
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
      <DesktopUpdate />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Change release channel" }));
  const dialog = await screen.findByRole("dialog", { name: "Change release channel" });
  fireEvent.click(within(dialog).getByRole("button", { name: "Use an earlier copy" }));
  invoke.mockRejectedValueOnce(
    "This Stable release can’t open this earlier copy of your data yet.",
  );
  fireEvent.click(
    within(dialog).getByRole("button", { name: "Check Stable compatibility" }),
  );
  await within(dialog).findByText(
    "This Stable release can’t open this earlier copy of your data yet.",
  );
  expect(
    within(dialog).queryByRole("button", { name: "Install Stable and restart" }),
  ).toBeNull();
  expect(
    within(dialog).getByRole("button", {
      name: "Keep checking for Nightly updates while waiting",
    }),
  ).toBeEnabled();

  invoke.mockResolvedValueOnce(null);
  fireEvent.click(
    within(dialog).getByRole("button", {
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
  expect(screen.getByText("Preview builds").closest("details")).not.toHaveAttribute(
    "open",
  );
  fireEvent.click(screen.getByText("Preview builds"));
  fireEvent.click(screen.getByRole("button", { name: "Try Nightly…" }));
  const dialog = await screen.findByRole("dialog", { name: "Change release channel" });
  expect(within(dialog).getAllByRole("button", { name: "Close" })).toHaveLength(1);
  expect(
    within(dialog).getByRole("button", { name: "Use my current data" }),
  ).toHaveAttribute("aria-pressed", "true");
  fireEvent.click(within(dialog).getByRole("button", { name: "Use a separate copy" }));
  expect(
    within(dialog).getByText(
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
  fireEvent.click(
    within(dialog).getByRole("button", { name: "Check Nightly compatibility" }),
  );
  const install = await within(dialog).findByRole("button", {
    name: "Install Nightly and restart",
  });
  expect(
    within(dialog).getByText("After switching, the app will use your existing data."),
  ).toBeVisible();
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
  expect(screen.getByText("Preview builds").closest("details")).not.toHaveAttribute(
    "open",
  );
  fireEvent.click(screen.getByText("Preview builds"));
  fireEvent.click(screen.getByRole("button", { name: "Try Nightly…" }));
  const dialog = await screen.findByRole("dialog", { name: "Change release channel" });
  expect(
    within(dialog).queryByRole("button", { name: "Use a separate copy" }),
  ).toBeNull();
  fireEvent.click(within(dialog).getByRole("button", { name: "Use the other copy" }));
  expect(
    within(dialog).getByText(
      "Reopens your other copy with its earlier changes. Neither copy is overwritten.",
    ),
  ).toBeVisible();
  invoke.mockRejectedValueOnce("Try again later.");
  fireEvent.click(
    within(dialog).getByRole("button", { name: "Check Nightly compatibility" }),
  );
  await within(dialog).findByText("Try again later.");
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
    fireEvent.click(screen.getByRole("button", { name: "Change release channel" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Change release channel",
    });
    expect(
      within(dialog).getByRole("button", { name: "Use my current data" }),
    ).toHaveAttribute("aria-pressed", "true");
    expect(
      within(dialog).getByText(
        "Keep the data you’ve been using in Nightly, including your latest changes.",
      ),
    ).toBeVisible();
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Use an earlier copy" }),
    );
    expect(
      within(dialog).getByText(
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
    fireEvent.click(
      within(dialog).getByRole("button", { name: "Check Stable compatibility" }),
    );
    expect(
      await within(dialog).findByText(
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
    fireEvent.click(screen.getByRole("button", { name: "Change release channel" }));
    const dialog = await screen.findByRole("dialog", {
      name: "Change release channel",
    });
    expect(
      within(dialog).getByText(`Waiting for a compatible Stable release. ${status}`),
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
  fireEvent.click(screen.getByRole("button", { name: "Change release channel" }));
  const dialog = await screen.findByRole("dialog", { name: "Change release channel" });
  expect(
    within(dialog).getByText(
      "The earlier copy of your data is missing. Choose your current data to continue.",
    ),
  ).toBeVisible();
  expect(
    within(dialog).getByRole("button", { name: "Check Stable compatibility" }),
  ).toBeDisabled();
  expect(
    within(dialog).getByRole("button", {
      name: "Stay on this build and wait for the next Stable release",
    }),
  ).toBeDisabled();
  expect(
    within(dialog).getByRole("button", { name: "Cancel return to Stable" }),
  ).toBeEnabled();
  fireEvent.click(within(dialog).getByRole("button", { name: "Use my current data" }));
  expect(
    within(dialog).queryByText(
      "The earlier copy of your data is missing. Choose your current data to continue.",
    ),
  ).toBeNull();
  expect(
    within(dialog).getByRole("button", { name: "Check Stable compatibility" }),
  ).toBeEnabled();
  invoke.mockResolvedValueOnce(null);
  fireEvent.click(
    within(dialog).getByRole("button", {
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

it("ignores a previous dialog's review after closing and reopening with current data", async () => {
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
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <DesktopReleaseChannel />
    </ConfigProvider>,
  );
  fireEvent.click(screen.getByRole("button", { name: "Change release channel" }));
  let dialog = await screen.findByRole("dialog", { name: "Change release channel" });
  fireEvent.click(within(dialog).getByRole("button", { name: "Use an earlier copy" }));
  fireEvent.click(
    within(dialog).getByRole("button", { name: "Check Stable compatibility" }),
  );
  fireEvent.click(within(dialog).getByRole("button", { name: "Close" }));
  await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
  fireEvent.click(screen.getByRole("button", { name: "Change release channel" }));
  dialog = await screen.findByRole("dialog", { name: "Change release channel" });
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
  expect(
    within(dialog).getByRole("button", { name: "Use my current data" }),
  ).toHaveAttribute("aria-pressed", "true");
  expect(
    within(dialog).queryByRole("button", { name: "Install Stable and restart" }),
  ).toBeNull();
  invoke.mockResolvedValueOnce({
    id: "current-copy",
    version: "0.2.0",
    workspace: "separate",
    will_seed: false,
    needs_setup: false,
    will_upgrade: false,
  });
  fireEvent.click(
    within(dialog).getByRole("button", { name: "Check Stable compatibility" }),
  );
  fireEvent.click(
    await within(dialog).findByRole("button", { name: "Install Stable and restart" }),
  );
  await waitFor(() =>
    expect(invoke).toHaveBeenLastCalledWith("desktop_channel_action", {
      token: "current-document",
      action: { kind: "install", token: "current-copy" },
    }),
  );
});
