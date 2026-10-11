import { expect, test, type Locator } from "@playwright/test";

// Run against the caller's real Atlas server and the maintained source artifact.
// These bounds test pane composition and readable coverage, not screenshot pixels.
for (const width of [1680, 820, 390]) {
  for (const similar of [false, true]) {
    test(`${similar ? "Similar" : "browse"} search keeps usable panes and coverage at ${width}px`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height: 1000 });
      await page.goto(
        similar ? "/search?similar=equipment-srd%3A7jolv0cuttvjI1JD" : "/search",
      );
      const filter = page.locator(".workspace-pane--filter");
      const results = page.locator(".workspace-pane--results");
      const detail = page.locator(".workspace-pane--detail");
      await expect(page.locator(".result-row").first()).toBeVisible();
      await expect(filter.locator(":scope > *")).toHaveCount(2);
      const body = filter.locator(":scope > .filter-panel");
      await expect(body.getByRole("textbox", { name: "Search records" })).toBeVisible();
      const coverage = results.locator(".result-pane-header > span").first();
      await expect(coverage).toHaveText(similar ? "Top matches" : /\d[\d,]* records/);
      if (similar) {
        const criterion = body.getByText("Similar to Wovenwood Shield (Minor)", {
          exact: true,
        });
        await expect(criterion).toBeVisible();
        const criterionBox = await bounds(criterion);
        const filterHeader = await bounds(filter.locator(":scope > .pane-header"));
        expect(criterionBox.y - (filterHeader.y + filterHeader.height)).toBeLessThan(
          60,
        );
      }

      const filterBox = await bounds(filter);
      const resultsBox = await bounds(results);
      const detailBox = await bounds(detail);
      expect(resultsBox.width).toBeGreaterThan(width === 820 ? 400 : 340);
      if (width === 1680) {
        expect(resultsBox.x).toBeGreaterThanOrEqual(filterBox.x + filterBox.width);
        expect(detailBox.x).toBeGreaterThanOrEqual(resultsBox.x + resultsBox.width);
        await expect(
          page.getByRole("separator", { name: "Resize filters" }),
        ).toBeVisible();
      } else {
        await expect(page.getByRole("separator")).toHaveCount(0);
        expect(detailBox.y).toBeGreaterThanOrEqual(
          Math.max(filterBox.y + filterBox.height, resultsBox.y + resultsBox.height) -
            1,
        );
        expect(detailBox.x).toBeCloseTo(filterBox.x, 0);
        if (width === 390) {
          expect(resultsBox.y).toBeGreaterThanOrEqual(filterBox.y + filterBox.height);
        } else {
          expect(resultsBox.y).toBeCloseTo(filterBox.y, 0);
          expect(resultsBox.x).toBeGreaterThanOrEqual(filterBox.x + filterBox.width);
        }
      }
      await coverage.scrollIntoViewIfNeeded();
      expect(
        await coverage.evaluate((element) => {
          const text = document.createRange();
          text.selectNodeContents(element);
          const content = text.getBoundingClientRect();
          const box = element.getBoundingClientRect();
          return (
            content.width <= box.width + 1 &&
            box.left >= 0 &&
            box.right <= window.innerWidth
          );
        }),
      ).toBe(true);
      const title = results.locator(":scope > .pane-header > span");
      expect(
        await title.evaluate((element) => element.scrollWidth <= element.clientWidth),
      ).toBe(true);
      for (const control of [
        results.getByRole("combobox", { name: "Records per page" }),
        results.getByRole("textbox", { name: "Current page" }),
        results.getByRole("button", { name: "Next page" }),
      ]) {
        const box = await bounds(control);
        expect(box.x).toBeGreaterThanOrEqual(resultsBox.x);
        expect(box.x + box.width).toBeLessThanOrEqual(resultsBox.x + resultsBox.width);
      }
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= window.innerWidth,
        ),
      ).toBe(true);
      await page.screenshot({
        path: `../../scratch/browser-validation/search-${similar ? "similar" : "browse"}-${width}.png`,
      });
    });
  }
}

test("responsive reflow preserves wide resize state and collapse controls", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1680, height: 1000 });
  await page.goto("/search");
  const filter = page.locator(".workspace-pane--filter");
  const handle = page.getByRole("separator", { name: "Resize filters" });
  await handle.focus();
  await handle.press("ArrowRight");
  await expect(handle).toHaveAttribute("aria-valuenow", "324");
  expect((await bounds(filter)).width).toBeCloseTo(324, 0);
  await page.setViewportSize({ width: 820, height: 1000 });
  expect((await bounds(filter)).width).toBeCloseTo(290, 0);
  await page.getByRole("button", { name: "Collapse Filters", exact: true }).click();
  expect((await bounds(filter)).width).toBeCloseTo(44, 0);
  expect(
    (await bounds(page.locator(".workspace-pane--results"))).width,
  ).toBeGreaterThan(700);
  await page
    .getByRole("button", { name: "Expand Filters", exact: true })
    .first()
    .click();
  await page.setViewportSize({ width: 1680, height: 1000 });
  expect((await bounds(filter)).width).toBeCloseTo(324, 0);
  await expect(handle).toBeVisible();
});

async function bounds(locator: Locator) {
  const box = await locator.boundingBox();
  expect(box).not.toBeNull();
  return box!;
}
