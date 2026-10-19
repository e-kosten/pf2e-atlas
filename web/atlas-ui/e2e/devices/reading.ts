import { expect, type Locator, type Page, type TestInfo } from "@playwright/test";
import { encodeFilterUrl } from "../../src/shared/filters/filterUrl";

export const sentinel = "browser-actors:sentinel00000001";
export const offGuard = "browser-conditions:offguard00000001";
export const landing = {
  record_key: sentinel,
  embedded_path: [{ collection: "/items", identity: { Stable: "signalpulse00001" } }],
  field: null,
  passage: null,
  source_fingerprint: null,
};
export const creatureFilter = encodeURIComponent(
  encodeFilterUrl({
    kind: "in",
    clause_id: "kind",
    field: "kind",
    values: ["creature"],
  }),
);
export const landedSearch = `/search/records/${sentinel}?filter=${creatureFilter}&landing=${encodeURIComponent(JSON.stringify(landing))}`;

export async function activate(control: Locator, info: TestInfo) {
  if (info.project.use.hasTouch) await control.tap();
  else await control.click();
}

export async function noPageOverflow(page: Page) {
  expect(
    await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth + 1),
  ).toBe(true);
}

export async function readThenBack(
  page: Page,
  host: Locator,
  info: TestInfo,
  targetName = "Off-Guard",
  returnMode: "back" | "close" = "back",
) {
  const link = host.getByRole("link", { name: targetName, exact: true }).last();
  await expect(link).toBeVisible();
  // Put the departing link in the actual scroll host, then capture the position
  // at activation time. Subsequent reflow is checked by anchor, not pixel identity.
  await link.scrollIntoViewIfNeeded();
  await link.focus();
  const position = await host.evaluate((node) => node.scrollTop);
  expect(position).toBeGreaterThan(0);
  const href = await link.getAttribute("href");
  // Keyboard activation avoids a pointer's pre-click scroll changing the bookmark.
  await link.press("Enter");
  const drawer = page.locator(".record-preview-drawer");
  const title = async (name: string | RegExp) => {
    if (await drawer.isVisible())
      await expect(page.getByRole("dialog", { name })).toBeVisible();
    else
      await expect(
        (returnMode === "close"
          ? page.locator(".record-preview-popover__body")
          : host
        ).getByRole("heading", { name }),
      ).toBeVisible();
  };
  await title(targetName);
  if (returnMode === "close") {
    const narrow = await drawer.isVisible();
    await activate(
      page.getByRole("button", {
        name: narrow ? "Close" : "Close reference preview",
        exact: true,
      }),
      info,
    );
    await expect(
      narrow ? drawer : page.locator(".record-preview-popover"),
    ).toBeHidden();
    await expect(host.getByRole("heading", { name: /^Signal Pulse\b/ })).toBeVisible();
  } else {
    await activate(page.getByRole("button", { name: "Back", exact: true }), info);
    await title(/^Signal Pulse\b/);
  }
  await expect
    .poll(() => host.evaluate((node) => node.scrollTop))
    .toBeCloseTo(position, 0);
  await expect(page.locator(":focus")).toHaveAttribute("href", href!);
  return position;
}
