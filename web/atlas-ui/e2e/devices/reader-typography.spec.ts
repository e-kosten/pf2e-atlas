import { devices, type Locator } from "@playwright/test";
import { expect, test } from "./fixtures";
import { activate } from "./reading";

const fontSize = (element: Locator) =>
  element.evaluate((node) => parseFloat(getComputedStyle(node).fontSize));

test.describe("narrow desktop tabs with touch input", () => {
  // Desktop-UA touch devices use the overflow menu; phone UA uses scrollable tabs.
  test.use({
    hasTouch: true,
    isMobile: false,
    userAgent: devices["Desktop Chrome"].userAgent,
  });

  test("Explore overflow dismisses after selecting Versions", async ({ page }) => {
    await page.setViewportSize({ width: 320, height: 844 });
    await page.goto("/records/browser-editions:remaster00000001");
    await page
      .getByRole("button", { name: "Explore related records", exact: true })
      .tap();
    const more = page.locator(".record-explore .ant-tabs-nav-more");
    await expect(more).toBeVisible();
    await more.tap();
    const menu = page.getByRole("listbox", { name: "expanded dropdown", exact: true });
    await menu.getByRole("option", { name: "Versions", exact: true }).tap();
    await expect(menu).toBeHidden();
    await expect(page.locator(".remaster-relationship__line")).toContainText(
      "Legacy signal",
    );
    await expect(
      page.getByRole("button", { name: "Compare", exact: true }),
    ).toBeVisible();
  });
});

test("reading and Versions share inline text size without enlarging controls", async ({
  page,
}, info) => {
  await page.goto("/records/browser-editions:remaster00000001");
  const prose = page.getByText("Modern signal instructions.", { exact: true });
  await expect(prose).toBeVisible();
  const bodySize = await fontSize(prose);
  const notice = page.locator(".record-edition-notice .remaster-relationship__line");
  await expect(notice).toBeVisible();
  expect(await fontSize(notice)).toBe(bodySize);
  expect(
    await fontSize(notice.getByRole("link", { name: "Legacy signal", exact: true })),
  ).toBe(bodySize);
  expect(
    await fontSize(page.getByRole("heading", { name: "Modern signal", exact: true })),
  ).toBeGreaterThan(bodySize);

  const controlSize = await fontSize(
    page.getByRole("button", { name: "Theme", exact: true }),
  );
  await activate(
    page.getByRole("button", { name: "Explore related records", exact: true }),
    info,
  );
  await activate(page.getByRole("tab", { name: "Versions", exact: true }), info);
  const relationship = page.locator(".remaster-relationship__line");
  await expect(relationship).toBeVisible();
  expect(await fontSize(relationship)).toBe(bodySize);
  expect(
    await fontSize(
      relationship.getByRole("link", { name: "Legacy signal", exact: true }),
    ),
  ).toBe(bodySize);
  const compare = page
    .locator(".remaster-relationship__actions")
    .getByRole("button", { name: "Compare", exact: true });
  await expect(compare).toBeVisible();
  await expect(
    relationship.getByRole("button", { name: "Compare", exact: true }),
  ).toHaveCount(0);
  expect(await fontSize(compare)).toBe(controlSize);
  const lineBounds = await relationship.boundingBox();
  const actionBounds = await compare.boundingBox();
  expect(lineBounds).not.toBeNull();
  expect(actionBounds).not.toBeNull();
  expect(actionBounds!.y).toBeGreaterThanOrEqual(lineBounds!.y + lineBounds!.height);
  expect(controlSize).toBeLessThan(bodySize);
});

test("reader links follow prose and badge links retain the badge scale", async ({
  page,
}) => {
  await page.goto("/records/browser-spells:spell00000000001");
  const prose = page
    .locator(".prepared-content")
    .filter({ hasText: "A warm light" })
    .first();
  await expect(prose).toBeVisible();
  const bodySize = await fontSize(prose);
  expect(
    await fontSize(prose.getByRole("link", { name: "Signal Protocol", exact: true })),
  ).toBe(bodySize);
  const trait = page
    .locator(".record-trait-badge")
    .filter({ has: page.getByRole("link", { name: "Light", exact: true }) })
    .first();
  const traitLink = trait.getByRole("link", { name: "Light", exact: true });
  await expect(traitLink).toBeVisible();
  const badgeSize = await fontSize(trait);
  expect(await fontSize(traitLink)).toBe(badgeSize);
  expect(badgeSize).toBeLessThan(bodySize);
});

test("a link at the edge of authored prose has room for its focus outline", async ({
  page,
}) => {
  await page.goto("/records/browser-rules:rule000000000001");
  const prose = page
    .locator(".prepared-content")
    .filter({ hasText: "The lantern flashes" });
  await prose.getByRole("link", { name: "Off-Guard", exact: true }).press("Tab");
  const link = prose.getByRole("link", { name: "Lantern Ward", exact: true });
  await expect(link).toBeFocused();
  await expect(link).toHaveCSS("outline-style", "solid");
  expect(
    await link.evaluate((element) => {
      const content = element.closest(".prepared-content")!;
      const { outlineWidth, outlineOffset } = getComputedStyle(element);
      const leftClearance =
        element.getBoundingClientRect().left - content.getBoundingClientRect().left;
      const width = parseFloat(outlineWidth);
      return (
        width > 0 &&
        (getComputedStyle(content).overflowX === "visible" ||
          leftClearance >= width + parseFloat(outlineOffset))
      );
    }),
  ).toBe(true);
});
