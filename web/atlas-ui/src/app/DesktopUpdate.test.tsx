import { ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { DesktopReleaseChannel, DesktopUpdate } from "./DesktopUpdate";

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
      active: "trial",
      trial_path: "/Atlas/Trial/nightly-trial.sqlite",
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
  fireEvent.click(screen.getByRole("button", { name: "Release channel" }));
  const dialog = await screen.findByRole("dialog", { name: "Atlas release channel" });
  fireEvent.click(within(dialog).getByRole("button", { name: "Primary workspace" }));
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
      action: { kind: "wait", workspace: "primary", updates: "hold" },
    }),
  );
});
