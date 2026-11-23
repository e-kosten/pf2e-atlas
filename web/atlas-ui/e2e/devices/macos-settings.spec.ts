import { expect, test } from "@playwright/test";

test("macOS Settings scrolls long content below the pane selector", async ({
  page,
}) => {
  await page.setViewportSize({ width: 420, height: 340 });
  await page.addInitScript(() => {
    const invoke = async (_command: string, args: { action?: string }) => {
      if (args.action === "read_terminal")
        return { state: "linked", version: "1.0.0", path: "/usr/local/bin/atlas" };
    };
    Object.assign(window, {
      __TAURI__: { core: { invoke } },
      __ATLAS_DESKTOP_SETUP__: {
        token: "settings-fixture",
        settingsWindow: true,
        setup: { job: 0, sequence: 0, status: "idle" },
      },
    });
  });
  await page.goto("/settings/terminal");
  await page.getByText("Command details", { exact: true }).click();
  const toolbar = page.getByRole("navigation", { name: "Settings panes" });
  const heading = page.getByRole("heading", { name: "Terminal command" });
  const lastAction = page.getByRole("button", { name: "Choose bin folder…" });
  const toolbarBottom = await toolbar.evaluate(
    (nav) => nav.getBoundingClientRect().bottom,
  );
  const scrollport = page.locator(".desktop-settings__content");
  expect(await scrollport.evaluate((pane) => pane.getBoundingClientRect().top)).toBe(
    toolbarBottom,
  );
  await expect(lastAction).not.toBeInViewport();
  await heading.hover();
  await page.mouse.wheel(0, 640);
  await expect(lastAction).toBeInViewport({ ratio: 1 });
  expect(await page.evaluate(() => window.scrollY)).toBe(0);
  expect(await toolbar.evaluate((nav) => nav.getBoundingClientRect().bottom)).toBe(
    toolbarBottom,
  );
  await toolbar.getByRole("link", { name: "Updates" }).click();
  await expect(page.getByText("Update information is unavailable.")).toBeInViewport();
});
