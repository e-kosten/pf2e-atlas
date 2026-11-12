import { ConfigProvider } from "antd";
import { fireEvent, render, screen, within } from "@testing-library/react";
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
    reference: {
      revision: "1111111111111111111111111111111111111111",
      target: "2222222222222222222222222222222222222222",
      targetVersion: "8.6.1",
      managed: true,
      available: true,
    },
    setup: { job: 2, sequence: 5, status: "idle" },
  };
  invoke.mockResolvedValue(undefined);
}

function renderPanel() {
  render(
    <ConfigProvider theme={{ token: { motion: false } }}>
      <DesktopSetupPanel />
    </ConfigProvider>,
  );
}

it("exposes workspace setup only for an attached desktop document", () => {
  renderPanel();
  expect(screen.queryByRole("button", { name: "Setup and Updates" })).toBeNull();
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
  expect(invoke).toHaveBeenCalledTimes(1);
  fireEvent.click(apply());
  expect(invoke).toHaveBeenLastCalledWith("desktop_setup_action", {
    action: "apply_reference",
    job: 2,
    sequence: 5,
    token: "current-document",
  });
});
