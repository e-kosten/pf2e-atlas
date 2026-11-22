import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createMemoryHistory, RouterProvider } from "@tanstack/react-router";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { DesktopTabs } from "./DesktopTabs";
import { createAtlasRouter } from "./router";
import type { DesktopAttachment } from "./desktopBridge";
import { getReadiness } from "../api/atlasApi";

vi.mock("../api/atlasApi", async (original) => ({
  ...(await original<typeof import("../api/atlasApi")>()),
  verifyBackendCompatibility: vi.fn().mockResolvedValue({}),
  getReadiness: vi.fn(),
}));

const desktop: DesktopAttachment = {
  token: "settings-test",
  setup: { job: 0, sequence: 1, status: "idle", local_build: false, can_resume: false },
  channel: {
    installed: { tag: "v1.0.0", version: "1.0.0" },
    active: "original",
    workspace_path: "/workspace",
    other_exists: false,
    returning: null,
    ready: false,
    busy: false,
  },
};
const invoke = vi.fn(async (command: string, args: { action?: string }) => {
  if (command === "desktop_setup_action" && args.action === "read_terminal")
    return { command: '& "C:\\Atlas\\atlas.exe"' };
});
function attach(attachment: DesktopAttachment, windows: boolean) {
  vi.stubGlobal("__ATLAS_DESKTOP_SETUP__", attachment);
  vi.stubGlobal("__ATLAS_WINDOWS_TABS__", windows);
  vi.stubGlobal("__TAURI__", { core: { invoke } });
  vi.stubGlobal("__TAURI_INTERNALS__", { invoke });
}
function open(path: string, windows: boolean) {
  vi.spyOn(window, "scrollTo").mockImplementation(() => {});
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      {windows ? (
        <DesktopTabs initialHref={path} />
      ) : (
        <RouterProvider
          router={createAtlasRouter(createMemoryHistory({ initialEntries: [path] }))}
        />
      )}
    </QueryClientProvider>,
  );
}
afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
  vi.clearAllMocks();
});

it("delivers an early request to a single Windows Settings tab and returns focus to the originating row", async () => {
  attach({ ...desktop, settingsRequest: { id: 1, destination: "terminal" } }, true);
  open("/settings", true);
  expect(await screen.findByText('& "C:\\Atlas\\atlas.exe"')).toBeVisible();
  expect(invoke).toHaveBeenCalledWith("desktop_settings_ack", {
    token: desktop.token,
    id: 1,
  });
  expect(screen.queryByRole("tablist")).toBeNull();
  fireEvent.click(screen.getByRole("link", { name: "Back to Settings" }));
  const terminal = await screen.findByRole("link", { name: "Terminal access" });
  await waitFor(() => expect(terminal).toHaveFocus());
  const channel = screen.getByRole("link", { name: /Release channel.*Stable/ });
  fireEvent.click(channel);
  expect(await screen.findByRole("heading", { name: "Release channel" })).toHaveFocus();
  fireEvent.click(screen.getByRole("link", { name: "Back to Settings" }));
  await waitFor(() =>
    expect(screen.getByRole("link", { name: /Release channel.*Stable/ })).toHaveFocus(),
  );
  expect(getReadiness).not.toHaveBeenCalled();
});

it("opens a targeted macOS channel review in the Updates pane and keeps the toolbar available", async () => {
  attach({ ...desktop, settingsWindow: true }, false);
  open("/settings/terminal", false);
  const nav = await screen.findByRole("navigation", { name: "Settings panes" });
  expect(within(nav).getByRole("link", { name: "Terminal" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await act(async () => {
    attach(
      {
        ...desktop,
        settingsWindow: true,
        settingsRequest: { id: 2, destination: "channel" },
      },
      false,
    );
    window.dispatchEvent(new Event("atlas-desktop-setup"));
  });
  expect(await screen.findByText("Current channel · Stable")).toBeVisible();
  expect(within(nav).getByRole("link", { name: "Updates" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  expect(invoke).toHaveBeenCalledWith("desktop_settings_ack", {
    token: desktop.token,
    id: 2,
  });
  await act(async () => {
    attach(
      {
        ...desktop,
        settingsWindow: true,
        settingsRequest: { id: 1, destination: "terminal" },
      },
      false,
    );
    window.dispatchEvent(new Event("atlas-desktop-setup"));
  });
  expect(screen.getByText("Current channel · Stable")).toBeVisible();
  fireEvent.click(screen.getByText("Release channel", { selector: "summary" }));
  await waitFor(() =>
    expect(screen.queryByText("Current channel · Stable")).toBeNull(),
  );
  expect(getReadiness).not.toHaveBeenCalled();
});
