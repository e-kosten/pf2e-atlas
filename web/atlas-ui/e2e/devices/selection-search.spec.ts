import type { Locator, Page, TestInfo } from "@playwright/test";
import { expect, test } from "./fixtures";
import { activate, landedSearch, noPageOverflow } from "./reading";

async function selectParagraph(page: Page, reading: Locator, info: TestInfo) {
  const paragraph = reading
    .locator(".prepared-content p")
    .filter({ hasText: /\S/ })
    .first();
  await expect(paragraph).toBeVisible();
  await paragraph.scrollIntoViewIfNeeded();
  if (info.project.use.hasTouch) {
    await paragraph.tap({ position: { x: 4, y: 8 } });
    // Emulated touch has no OS text-selection handles. Exercise the DOM selection
    // lifecycle and touch activation; actual-device long press remains separate.
    await paragraph.evaluate((element) => {
      const range = document.createRange();
      range.selectNodeContents(element);
      const selection = window.getSelection()!;
      selection.removeAllRanges();
      selection.addRange(range);
      document.dispatchEvent(new Event("selectionchange"));
    });
  } else await paragraph.click({ clickCount: 3, position: { x: 4, y: 8 } });
  const action = page.getByRole("link", { name: "Search Atlas", exact: true });
  await expect(action).toBeVisible();
  const text = await page.evaluate(() =>
    window.getSelection()!.toString().replace(/\s+/g, " ").trim(),
  );
  expect(text).not.toBe("");
  const bounds = await action.boundingBox();
  expect(bounds).not.toBeNull();
  const viewport = page.viewportSize()!;
  expect(bounds!.x).toBeGreaterThanOrEqual(0);
  expect(bounds!.y).toBeGreaterThanOrEqual(0);
  expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(viewport.width);
  expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(viewport.height);
  const url = new URL((await action.getAttribute("href"))!, page.url());
  expect(url.pathname).toBe("/search");
  expect(Object.fromEntries(url.searchParams)).toEqual({
    q: text,
    search_profile: "bge-base",
  });
  return { action, text, href: url.href };
}

async function expectFreshSearch(page: Page, text: string) {
  await expect(page).toHaveURL(
    (url) =>
      url.pathname === "/search" &&
      url.searchParams.get("q") === text &&
      !url.searchParams.has("filter"),
  );
  await expect(page.getByRole("dialog")).toHaveCount(0);
  await expect(
    page.getByRole("searchbox", { name: "Search records", exact: true }),
  ).toHaveValue(text);
  await noPageOverflow(page);
}

test("selected text offers a visible keyboard action and starts a fresh search", async ({
  page,
}, info) => {
  await page.goto("/records/browser-conditions:offguard00000001");
  const article = page.locator("article[data-selection-search]");
  const originalUrl = page.url();
  const first = await selectParagraph(page, article, info);
  expect(page.url()).toBe(originalUrl);
  await page.keyboard.press("Tab");
  await expect(first.action).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(first.action).toHaveCount(0);
  expect(
    await page.evaluate(() =>
      window.getSelection()?.toString().replace(/\s+/g, " ").trim(),
    ),
  ).toBe(first.text);
  const selected = await selectParagraph(page, article, info);
  await activate(selected.action, info);
  await expectFreshSearch(page, selected.text);
});

test("linked-preview search preserves the reader in a new tab and closes it for same-tab search", async ({
  page,
}, info) => {
  await page.goto(landedSearch);
  const host = page.locator(".reference-reader__scroll");
  await activate(
    host.getByRole("link", { name: "Off-Guard", exact: true }).last(),
    info,
  );
  const drawer = page.getByRole("dialog", { name: "Off-Guard", exact: true });
  await expect(drawer).toBeVisible();
  const originalUrl = page.url();
  let selected = await selectParagraph(page, drawer, info);
  const opened = page.context().waitForEvent("page");
  await selected.action.click({ modifiers: ["ControlOrMeta"] });
  const child = await opened;
  await child.waitForLoadState();
  expect(new URL(child.url()).searchParams.get("q")).toBe(selected.text);
  await child.close();
  expect(page.url()).toBe(originalUrl);
  await expect(drawer).toBeVisible();
  selected = await selectParagraph(page, drawer, info);
  await activate(selected.action, info);
  await expectFreshSearch(page, selected.text);
});

test("selected-text search leaves a retained Links picker", async ({ page }, info) => {
  await page.goto("/search");
  await activate(page.getByRole("button", { name: "Links", exact: true }), info);
  const picker = page.getByRole("dialog", { name: "Add link condition", exact: true });
  await expect(picker).toBeVisible();
  const row = picker.locator(".reference-result").first().getByRole("link").first();
  await activate(row, info);
  const selected = await selectParagraph(page, picker, info);
  await activate(selected.action, info);
  await expectFreshSearch(page, selected.text);
});
