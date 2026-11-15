import { devices, type Locator } from "@playwright/test";
import { expect, test } from "./fixtures";
import { activate, noPageOverflow } from "./reading";

const fontSize = (element: Locator) =>
  element.evaluate((node) => parseFloat(getComputedStyle(node).fontSize));

async function expectChipTypography(chip: Locator) {
  await expect(chip).toHaveCSS("font-size", "12px");
  await expect(chip).toHaveCSS("font-weight", "400");
  await expect(chip).toHaveCSS("line-height", "20px");
}

test.describe("narrow desktop tabs with touch input", () => {
  // Desktop-UA touch devices use the overflow menu; phone UA uses scrollable tabs.
  test.use({
    hasTouch: true,
    isMobile: false,
    userAgent: devices["Desktop Chrome"].userAgent,
  });

  test("Explore keeps Versions visible and dismisses overflow after selecting Similar", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 320, height: 844 });
    await page.goto("/records/browser-editions:remaster00000001");
    await page
      .getByRole("button", { name: "Explore related records", exact: true })
      .tap();
    const versions = page.getByRole("tab", { name: "Versions", exact: true });
    await expect(versions).toBeInViewport({ ratio: 1 });
    await expect(versions).toHaveAttribute("aria-selected", "true");
    await expect(page.locator(".remaster-relationship__line")).toContainText(
      "Legacy signal",
    );
    await expect(
      page.getByRole("button", { name: "Compare", exact: true }),
    ).toBeVisible();
    const more = page.locator(".record-explore .ant-tabs-nav-more");
    await expect(more).toBeVisible();
    await more.tap();
    const menu = page.getByRole("listbox", { name: "expanded dropdown", exact: true });
    await menu.getByRole("option", { name: "Similar", exact: true }).tap();
    await expect(menu).toBeHidden();
    await expect(
      page.getByRole("tab", { name: "Similar", selected: true }),
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
  await expectChipTypography(traitLink);
  const staticChip = trait.locator("..").locator(".display-chip:not(:has(a))").first();
  await expectChipTypography(staticChip);
  expect(await fontSize(traitLink)).toBeLessThan(bodySize);
});

test("drawer chip links match static chips and restore keyboard focus", async ({
  page,
}) => {
  await page.goto("/records/browser-rules:rule000000000001");
  await page.getByRole("link", { name: "Lantern Ward", exact: true }).press("Enter");

  const drawer = page.locator(".record-preview-drawer-root").getByRole("dialog");
  const heading = drawer.getByRole("heading", { name: "Lantern Ward", exact: true });
  await expect(heading).toBeVisible();
  const header = heading.locator("xpath=ancestor::header");
  const trait = header.getByRole("link", { name: "Light", exact: true });
  const staticChip = header.locator(".display-chip:not(:has(a))").first();

  await expectChipTypography(trait);
  await expectChipTypography(staticChip);
  // Set keyboard modality before focusing a link on touch-oriented WebKit.
  await page.keyboard.press("Tab");
  await trait.focus();
  await expect(trait).toBeFocused();
  await expect(trait).toHaveCSS("outline-style", "solid");
  await trait.press("Enter");
  await expect(
    drawer.getByRole("heading", { name: "Light", exact: true }),
  ).toBeVisible();
  await drawer.getByRole("button", { name: "Back", exact: true }).press("Enter");
  await expect(heading).toBeVisible();
  await expect(trait).toBeFocused();
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

test("a remaster annotation keeps its target beside the linked label at narrow widths", async ({
  page,
}, info) => {
  await page.goto("/records/browser-editions:remaster00000001");
  const notice = page.locator(".record-edition-notice");
  const label = notice.getByRole("link", { name: "Legacy signal", exact: true });
  const annotation = notice.getByRole("link", {
    name: "Remaster Changes — Class Features",
    exact: true,
  });
  await expect(annotation).toBeVisible();
  for (const width of [320, 390]) {
    await page.setViewportSize({ width, height: 844 });
    const text = await label.boundingBox();
    const target = await annotation.boundingBox();
    expect(text).not.toBeNull();
    expect(target).not.toBeNull();
    expect(target!.width).toBeGreaterThanOrEqual(24);
    expect(target!.height).toBeGreaterThanOrEqual(24);
    expect(target!.x).toBeGreaterThanOrEqual(text!.x + text!.width - 0.5);
    expect(target!.y).toBeLessThan(text!.y + text!.height);
    expect(target!.y + target!.height).toBeGreaterThan(text!.y);
    await noPageOverflow(page);
  }
  await activate(annotation, info);
  await expect(
    page.getByRole("heading", { name: "Class Features", exact: true }).first(),
  ).toBeVisible();
});
