import { ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
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

  invoke.mockRejectedValueOnce("Atlas couldn’t verify the update. Try again.");
  fireEvent.click(within(dialog).getByRole("button", { name: "Install update" }));
  await within(dialog).findByText("Atlas couldn’t verify the update. Try again.");
  expect(invoke).toHaveBeenLastCalledWith("desktop_update_action", {
    token: "current-document",
    version: "0.2.3",
    action: "install",
  });
  expect(within(dialog).getByRole("button", { name: "Try again" })).toBeEnabled();

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
  fireEvent.click(
    within(dialog).getByRole("button", { name: "Return to my original workspace" }),
  );
  invoke.mockRejectedValueOnce("Stable cannot open this workspace yet.");
  fireEvent.click(
    within(dialog).getByRole("button", { name: "Check Stable compatibility" }),
  );
  await within(dialog).findByText("Stable cannot open this workspace yet.");
  expect(
    within(dialog).queryByRole("button", { name: "Install Stable and restart" }),
  ).toBeNull();
  expect(
    within(dialog).getByRole("button", {
      name: "Keep receiving Nightly updates while waiting",
    }),
  ).toBeEnabled();

  invoke.mockResolvedValueOnce(null);
  fireEvent.click(
    within(dialog).getByRole("button", {
      name: "Stay on this build and wait for Stable",
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
  fireEvent.click(screen.getByRole("button", { name: "Change release channel" }));
  const dialog = await screen.findByRole("dialog", { name: "Change release channel" });
  expect(within(dialog).getAllByRole("button", { name: "Close" })).toHaveLength(1);
  expect(
    within(dialog).getByRole("button", { name: "Continue with my current workspace" }),
  ).toHaveAttribute("aria-pressed", "true");
  fireEvent.click(within(dialog).getByRole("button", { name: "Use a separate copy" }));
  expect(
    within(dialog).getByText(
      "Starts with a copy of your current workspace. Changes stay separate from the original.",
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
    within(dialog).getByText(
      "After switching, the app and its bundled CLI will use this workspace.",
    ),
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
  fireEvent.click(screen.getByRole("button", { name: "Change release channel" }));
  const dialog = await screen.findByRole("dialog", { name: "Change release channel" });
  expect(
    within(dialog).queryByRole("button", { name: "Use a separate copy" }),
  ).toBeNull();
  fireEvent.click(
    within(dialog).getByRole("button", { name: "Use my other workspace" }),
  );
  expect(
    within(dialog).getByText(
      "Reopens your other workspace with its earlier changes. Neither workspace is overwritten.",
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
