import { expect, type Locator, type Page, type TestInfo } from "@playwright/test";
import { encodeFilterUrl } from "../../src/shared/filters/filterUrl";

export const sentinel = "browser-actors:sentinel00000001";
export const offGuard = "browser-conditions:offguard00000001";
export const landing = {
  record_key: sentinel,
  owners: [{ collection: "/items", identity: { Stable: "signalpulse00001" } }],
  field: null,
  passage: null,
  source_fingerprint: null,
};
export const creatureFilter = encodeURIComponent(
  encodeFilterUrl({
    kind: "in",
    clause_id: "kind",
    field: "record.kind",
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

export async function readThenBack(page: Page, host: Locator, info: TestInfo) {
  const link = host.getByRole("link", { name: "Off-Guard", exact: true }).last();
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
  await expect(
    host.getByRole("heading", { name: "Off-Guard", exact: true }),
  ).toBeVisible();
  await activate(page.getByRole("button", { name: "Back", exact: true }), info);
  await expect(host.getByRole("heading", { name: /^Signal Pulse\b/ })).toBeVisible();
  await expect
    .poll(() => host.evaluate((node) => node.scrollTop))
    .toBeCloseTo(position, 0);
  await expect(page.locator(":focus")).toHaveAttribute("href", href!);
  return position;
}
