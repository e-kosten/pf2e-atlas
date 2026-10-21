import { expect, test } from "@playwright/test";
import { encodeFilterUrl } from "../src/shared/filters/filterUrl";

// These checks use the caller's real source artifact and application service.
for (const width of [1440, 1100, 390]) {
  for (const similar of [false, true]) {
    test(`${similar ? "Similar" : "browse"} uses a usable result browser at ${width}px`, async ({
      page,
    }) => {
      await page.setViewportSize({ width, height: 1000 });
      await page.goto(
        similar
          ? "/search?similar=equipment-srd%3A7jolv0cuttvjI1JD"
          : "/search?filter=" +
              encodeURIComponent(
                encodeFilterUrl({
                  kind: "in",
                  clause_id: "kind",
                  field: "kind",
                  values: ["creature"],
                }),
              ),
      );
      const results = page.getByRole("region", { name: "Search results", exact: true });
      const row = page.locator("[data-record-key]").first();
      await expect(row).toBeVisible();
      const search = page.getByRole("searchbox", { name: "Search records" });
      if (similar) {
        await expect(search).toHaveCount(0);
        await expect(page.getByRole("group", { name: "Similar search" })).toBeVisible();
      } else await expect(search).toBeVisible();
      const inputBox = await page.locator(".reference-search-entry").boundingBox();
      expect(inputBox).not.toBeNull();
      const controlHeightBox = similar
        ? await page
            .getByRole("button", { name: "Clear similar criterion" })
            .boundingBox()
        : inputBox;
      const primaryControls = page.locator(
        ".search-model-select, .filter-toolbar .common-choice-trigger, .filter-panel-actions button",
      );
      for (const control of await primaryControls.all()) {
        const controlBox = await control.boundingBox();
        expect(controlBox).not.toBeNull();
        expect(
          Math.abs(controlBox!.height - controlHeightBox!.height),
        ).toBeLessThanOrEqual(1);
      }
      const coverage = page.locator(".result-pane-header > span").first();
      await expect(coverage).toHaveText(similar ? /Top matches/ : /\d[\d,]* records/);
      if (similar) {
        const title = page
          .getByRole("group", { name: "Similar search" })
          .getByText("Wovenwood Shield (Minor)", { exact: true });
        await expect(title).toBeVisible();
        expect(
          await title.evaluate((element) => {
            const selection = window.getSelection()!;
            const range = document.createRange();
            range.selectNodeContents(element);
            selection.removeAllRanges();
            selection.addRange(range);
            const text = selection.toString();
            selection.removeAllRanges();
            return text;
          }),
        ).toBe("Wovenwood Shield (Minor)");
      }
      for (const control of [
        results.getByRole("combobox", { name: "Records per page" }),
        results.getByRole("textbox", { name: "Current page" }),
        results.getByRole("button", { name: "Next page" }),
      ]) {
        const box = await control.boundingBox();
        expect(box).not.toBeNull();
        expect(box!.x).toBeGreaterThanOrEqual(0);
        expect(box!.x + box!.width).toBeLessThanOrEqual(width);
      }
      await row.click();
      const reader = page.getByRole("region", { name: "Record preview", exact: true });
      await expect(reader.locator(".record-reference__title")).toBeVisible();
      const box = await reader.boundingBox();
      expect(box!.width).toBeGreaterThan(width === 390 ? 340 : 600);
      if (width === 390) {
        await expect(
          page.getByRole("searchbox", { name: "Search records" }),
        ).toBeHidden();
        await reader.getByRole("button", { name: "Results", exact: true }).click();
        await expect(row).toBeFocused();
        await expect(row).toBeVisible();
      } else {
        const resultBox = await results.boundingBox();
        expect(box!.x).toBeGreaterThanOrEqual(resultBox!.x + resultBox!.width);
      }
      expect(
        await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth),
      ).toBe(true);
      if (similar) {
        await page.getByRole("button", { name: "Clear similar criterion" }).click();
        await expect(search).toBeFocused();
        expect(new URL(page.url()).searchParams.has("similar")).toBe(false);
      }
    });
  }
}

test("Back keeps page, selected result and nonzero list/reader positions", async ({
  page,
}) => {
  await page.goto("/search?q=fire&mode=text_search");
  await page.locator("[data-record-key]").first().waitFor();
  await page.getByRole("button", { name: "Next page", exact: true }).click();
  await expect(page.getByRole("textbox", { name: "Current page" })).toHaveValue("2");
  const row = page.locator("[data-record-key]").nth(4);
  await row.click();
  await expect(
    page.locator(".reference-reader__scroll .record-reference__title"),
  ).toBeVisible();
  await page.locator(".reference-results__scroll").evaluate((e) => (e.scrollTop = 200));
  const selected = await row.getAttribute("data-record-key");
  await page.getByRole("link", { name: "Open full page", exact: true }).click();
  await expect(page).toHaveURL(/\/records\//);
  await page.goBack();
  await expect(page.getByRole("textbox", { name: "Current page" })).toHaveValue("2");
  await expect(page.locator("[data-record-key][aria-current=true]")).toHaveAttribute(
    "data-record-key",
    selected!,
  );
  await expect
    .poll(() => page.locator(".reference-results__scroll").evaluate((e) => e.scrollTop))
    .toBe(200);
  // Use a long creature reader to establish actual nonzero reader continuity.
  const search = page.getByRole("searchbox", { name: "Search records", exact: true });
  await search.fill("Imeckus Stroon");
  await search.press("Enter");
  await expect(page.getByRole("textbox", { name: "Current page" })).toHaveValue("1");
  await expect(page.locator(".reference-results__scroll")).toHaveAttribute(
    "aria-busy",
    "false",
  );
  await page.locator("[data-record-key]").filter({ hasText: "Imeckus Stroon" }).click();
  await expect(
    page.getByRole("heading", { name: "Imeckus Stroon", exact: true }),
  ).toBeVisible();
  const scroll = page.locator(".reference-reader__scroll");
  await scroll.evaluate((e) => (e.scrollTop = 400));
  await expect.poll(() => scroll.evaluate((e) => e.scrollTop)).toBe(400);
  await page.getByRole("link", { name: "Open full page", exact: true }).click();
  await page.goBack();
  await expect.poll(() => scroll.evaluate((e) => e.scrollTop)).toBe(400);
  // Back remounts the browser without its original preview-trigger ref.
  await page.setViewportSize({ width: 390, height: 1000 });
  await page.getByRole("button", { name: "Results", exact: true }).click();
  await expect(page.locator('[data-record-key][aria-current="true"]')).toBeFocused();
});

test("match popovers retain row layout and land inline without scrolling the app chrome", async ({
  page,
}) => {
  await page.goto("/search?q=fire&mode=text_search");
  const row = page.locator(".reference-result").filter({
    has: page.locator("[data-record-key]").filter({ hasText: "Imeckus Stroon" }),
  });
  const trigger = row.getByRole("button", {
    name: "3 matches in Imeckus Stroon",
    exact: true,
  });
  await expect(trigger).toBeVisible();
  const before = await row.boundingBox();
  await trigger.click();
  const popup = page.locator(".reference-match-list:visible");
  await expect(
    popup.getByRole("link", { name: "Staff of Fire (Major)", exact: true }),
  ).toBeFocused();
  expect((await row.boundingBox())!.height).toBe(before!.height);
  await page.keyboard.press("Escape");
  await expect(trigger).toBeFocused();
  await trigger.click();
  await popup.getByRole("link", { name: "Staff of Fire (Major)", exact: true }).click();
  await expect(popup).toBeHidden();
  await expect(page.locator(".reference-landing-highlight")).toContainText(
    "Staff of Fire",
  );
  expect(await page.evaluate(() => scrollY)).toBe(0);
  await expect(
    page.getByRole("heading", { name: "Imeckus Stroon", exact: true }),
  ).toHaveCount(1);
  await trigger.click();
  await popup.getByRole("link", { name: "Fire Shield", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Fire Shield", exact: true }),
  ).toBeVisible();
  await expect(page.locator(".reference-landing-highlight")).toHaveCount(0);
  await expect(
    page.getByRole("link", { name: "Return to Imeckus Stroon", exact: true }),
  ).toBeVisible();
});
