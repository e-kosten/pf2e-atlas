import { ConfigProvider } from "antd";
import { fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { DesktopUpdate } from "./DesktopUpdate";

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
