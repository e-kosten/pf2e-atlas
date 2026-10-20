import { expect, test } from "./fixtures";
import { creatureFilter, landedSearch, noPageOverflow, sentinel } from "./reading";

test("an open reference and unfinished query survive crossing the reader breakpoint", async ({
  page,
}) => {
  await page.goto(landedSearch);
  const input = page.getByRole("searchbox", { name: "Search records" });
  await input.fill("unfinished search intent");
  await expect(page.getByRole("heading", { name: /^Signal Pulse\b/ })).toBeVisible();
  await page.setViewportSize({ width: 390, height: 664 });
  await expect(page.getByRole("heading", { name: /^Signal Pulse\b/ })).toBeVisible();
  await page.getByRole("button", { name: "Results", exact: true }).click();
  await expect(input).toHaveValue("unfinished search intent");
  await expect(page.locator(`[data-record-key="${sentinel}"]`)).toBeFocused();
  await page.setViewportSize({ width: 1100, height: 700 });
  await expect(input).toHaveValue("unfinished search intent");
  await expect(page.getByRole("heading", { name: /^Signal Pulse\b/ })).toBeVisible();
  await noPageOverflow(page);
});

test("320 CSS pixel reflow keeps keyboard filters, results and the reader operable", async ({
  page,
}) => {
  // Effective-layout-width coverage for zoom/reflow. This is not physical-device
  // or browser text-enlargement evidence; those require the documented visual pass.
  await page.setViewportSize({ width: 320, height: 640 });
  await page.goto(`/search?filter=${creatureFilter}`);
  const kind = page.getByRole("button", { name: "Edit Kind filter", exact: true });
  await page.keyboard.press("Tab");
  // Walk native tab order rather than proving only programmatic focusability.
  for (
    let step = 0;
    step < 20 && !(await kind.evaluate((node) => node === document.activeElement));
    step++
  ) {
    await page.keyboard.press("Tab");
  }
  await expect(kind).toBeFocused();
  await page.keyboard.press("Enter");
  const drawer = page.getByRole("dialog", { name: "Kind", exact: true });
  await expect(drawer).toBeVisible();
  await page.keyboard.press("Escape");
  await expect(kind).toBeFocused();
  const row = page.locator(`[data-record-key="${sentinel}"]`);
  await row.click();
  await expect(
    page.getByRole("heading", { name: "Atlas Sentinel", exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Results", exact: true }).click();
  await expect(row).toBeFocused();
  await noPageOverflow(page);
});

test("missing query assets leave a repairable text search and usable filter browsing", async ({
  page,
}) => {
  await page.goto(`/search?filter=${creatureFilter}`);
  await expect(page.locator("[data-record-key]").first()).toBeVisible();
  await page
    .getByRole("searchbox", { name: "Search records" })
    .fill("lantern sentinel");
  await page.getByRole("searchbox", { name: "Search records" }).press("Enter");
  await expect(
    page.getByRole("alert").filter({ hasText: "Text search is unavailable" }),
  ).toBeVisible();
  await expect(page.getByRole("searchbox", { name: "Search records" })).toHaveValue(
    "lantern sentinel",
  );
  await page.getByRole("button", { name: "Clear filters", exact: true }).click();
  await expect(page.getByRole("searchbox", { name: "Search records" })).toHaveValue(
    "lantern sentinel",
  );
  await page.getByRole("searchbox", { name: "Search records" }).fill("");
  await page.getByRole("searchbox", { name: "Search records" }).press("Enter");
  await page.getByRole("button", { name: "Edit Kind filter", exact: true }).click();
  await page.getByRole("button", { name: /^Kind: Creature, neutral;/ }).click();
  await page.keyboard.press("Escape");
  await expect(page.locator("[data-record-key]").first()).toBeVisible();
});
test("short landscape filter popovers keep their footer reachable by pointer", async ({
  page,
}) => {
  await page.setViewportSize({ width: 844, height: 390 });
  await page.goto("/search");
  await page.getByRole("button", { name: "Edit Traits filter", exact: true }).click();
  const menu = page.getByRole("dialog", { name: "Traits choices", exact: true });
  const footer = menu.getByRole("checkbox", { name: "Show unavailable" });
  await expect(footer).toBeVisible();
  await expect
    .poll(async () => {
      const box = await menu.boundingBox();
      return !!box && box.y >= 0 && box.y + box.height <= 390;
    })
    .toBe(true);
  // Click at the current coordinates, without Playwright scrolling it into view.
  const box = await footer.boundingBox();
  expect(box).not.toBeNull();
  await page.mouse.click(box!.x + box!.width / 2, box!.y + box!.height / 2);
  await expect(footer).toBeChecked();
});
