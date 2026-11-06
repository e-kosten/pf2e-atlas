import type { Locator } from "@playwright/test";
import { expect, test } from "./fixtures";
import { activate } from "./reading";

const fontSize = (element: Locator) =>
  element.evaluate((node) => parseFloat(getComputedStyle(node).fontSize));

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
  expect(await fontSize(relationship.getByRole("button", { name: "Compare" }))).toBe(
    controlSize,
  );
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
    .filter({ hasText: "Concentrate" })
    .first();
  await expect(trait).toBeVisible();
  const badgeSize = await fontSize(trait);
  expect(
    await fontSize(trait.getByRole("link", { name: "Concentrate", exact: true })),
  ).toBe(badgeSize);
  expect(badgeSize).toBeLessThan(bodySize);
});
