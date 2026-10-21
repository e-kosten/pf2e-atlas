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
      const search = page.getByRole("searchbox", {
        name: similar ? "Similar search" : "Search records",
      });
      await expect(search).toBeVisible();
      if (similar) await expect(search).toHaveAttribute("readonly");
      const inputBox = await page.locator(".reference-search-entry").boundingBox();
      expect(inputBox).not.toBeNull();
      const controlHeightBox = inputBox;
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
        await expect(search).toHaveValue("Wovenwood Shield (Minor)");
        await search.focus();
        await search.press("ControlOrMeta+A");
        expect(
          await search.evaluate((element: HTMLInputElement) =>
            element.value.slice(element.selectionStart ?? 0, element.selectionEnd ?? 0),
          ),
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
        await expect(search).toBeHidden();
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
        const geometry = async () =>
          Promise.all([
            page.locator(".reference-search-entry").boundingBox(),
            page
              .getByRole("combobox", { name: "Search model" })
              .locator("..")
              .boundingBox(),
            page
              .getByRole("combobox", { name: "Search ranking" })
              .locator("..")
              .boundingBox(),
          ]);
        const before = await geometry();
        const nativeInput = await search.elementHandle();
        await page.getByRole("button", { name: "Clear similar criterion" }).click();
        const normalSearch = page.getByRole("searchbox", { name: "Search records" });
        await expect(normalSearch).toBeFocused();
        expect(
          await normalSearch.evaluate(
            (element, previous) => element === previous,
            nativeInput,
          ),
        ).toBe(true);
        expect(new URL(page.url()).searchParams.has("similar")).toBe(false);
        const after = await geometry();
        for (let index = 0; index < before.length; index++) {
          for (const dimension of ["x", "y", "width", "height"] as const) {
            expect(
              Math.abs(before[index]![dimension] - after[index]![dimension]),
            ).toBeLessThanOrEqual(1);
          }
        }
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

test("relationship scope stays tappable within a phone viewport and preserves exact selection", async ({
  browser,
  baseURL,
}) => {
  const context = await browser.newContext({
    baseURL,
    viewport: { width: 390, height: 844 },
    hasTouch: true,
    isMobile: true,
    colorScheme: "dark",
  });
  const page = await context.newPage();
  try {
    const constraint = {
      direction: "incoming",
      selection: {
        record_key: "conditionitems:dfCMdR4wnpbYNTix",
        embedded_path: [],
        field: null,
        passage: null,
        source_fingerprint: null,
      },
      include_contents: true,
    };
    await page.goto(
      "/search?relationships=" +
        encodeURIComponent("1." + JSON.stringify([constraint])),
    );
    const criterion = page.getByRole("group", { name: "Relationship filter" });
    await expect(criterion).toContainText("Stunned");
    const trigger = criterion.getByRole("button", { name: /Links to Stunned/ });
    await expect(trigger).toHaveText("Links to Stunned");
    await trigger.tap();
    const popover = page
      .getByRole("tooltip")
      .filter({ hasText: "Look for links in matching records" });
    await expect(popover).toBeVisible();
    const main = popover.getByRole("radio", { name: "Main entry only" });
    const whole = popover.getByRole("radio", { name: "Whole entry" });
    for (const control of [popover, main, whole]) {
      const box = await control.boundingBox();
      expect(box).not.toBeNull();
      expect(box!.x).toBeGreaterThanOrEqual(0);
      expect(box!.y).toBeGreaterThanOrEqual(0);
      expect(box!.x + box!.width).toBeLessThanOrEqual(390);
      expect(box!.y + box!.height).toBeLessThanOrEqual(844);
    }
    await main.tap();
    await expect(trigger).toHaveText("Links to Stunned");
    await expect(trigger).toBeFocused();
    const restored = JSON.parse(
      new URL(page.url()).searchParams.get("relationships")!.slice(2),
    );
    expect(restored).toEqual([{ ...constraint, include_contents: false }]);
    await trigger.tap();
    await expect(main).toBeChecked();
    await page.keyboard.press("Escape");
    await expect(trigger).toHaveAttribute("aria-expanded", "false");
    await expect(trigger).toBeFocused();
    await trigger.tap();
    await popover.getByRole("link", { name: "Open Stunned", exact: true }).tap();
    await expect(page).toHaveURL(
      (url) =>
        decodeURIComponent(url.pathname) === "/records/conditionitems:dfCMdR4wnpbYNTix",
    );
    expect(JSON.parse(new URL(page.url()).searchParams.get("selection")!)).toEqual(
      constraint.selection,
    );
    await expect(
      page.getByRole("heading", { name: "Stunned", exact: true }),
    ).toBeVisible();
  } finally {
    await context.close();
  }
});
