import { expect, test } from "@playwright/test";

test("measured tags hide immediately and their phone disclosure stays in the viewport", async ({
  page,
}) => {
  const slug = `browser-metadata-${Date.now()}`;
  const name = `Browser metadata ${slug}`;
  const tags = Array.from(
    { length: 20 },
    (_, index) => `review tag ${String(index + 1).padStart(2, "0")}`,
  );
  const created = await page.request.post("/api/lists", {
    data: { slug, name, tags, description: "Temporary browser regression fixture." },
  });
  expect(created.ok()).toBeTruthy();
  const { list } = await created.json();
  try {
    await page.addInitScript(() => {
      const evidence = { visibleHiddenTags: [] as string[] };
      Object.assign(window, { atlasTagMeasurementEvidence: evidence });
      new MutationObserver(() => {
        for (const tag of document.querySelectorAll<HTMLElement>(
          '[data-list-tag][aria-hidden="true"]',
        )) {
          if (getComputedStyle(tag).visibility !== "hidden")
            evidence.visibleHiddenTags.push(tag.textContent ?? "");
        }
      }).observe(document, {
        subtree: true,
        childList: true,
        attributes: true,
        attributeFilter: ["style", "aria-hidden"],
      });
    });
    await page.goto("/lists");
    await page.getByRole("textbox", { name: "Search saved lists" }).fill(name);
    const summary = page.getByLabel(name + " tags", { exact: true });
    await expect(summary.getByRole("button", { name: /^\+\d+ more$/ })).toBeVisible();
    const earlyVisibility = await page.evaluate(
      () =>
        (
          window as typeof window & {
            atlasTagMeasurementEvidence: { visibleHiddenTags: string[] };
          }
        ).atlasTagMeasurementEvidence.visibleHiddenTags,
    );
    expect(earlyVisibility).toEqual([]);

    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(`/lists/${list.slug}`);
    const trigger = page.getByRole("button", { name: /^\+\d+ more$/ });
    await trigger.click();
    const content = page.getByLabel("All list tags", { exact: true });
    const popup = page.getByRole("tooltip").filter({ has: content });
    await expect(popup).toBeVisible();
    await expect
      .poll(async () => {
        const box = await popup.boundingBox();
        return box !== null && box.x >= -1 && box.x + box.width <= 391;
      })
      .toBe(true);
    await expect(content.locator("span")).toHaveCount(tags.length);
    await page.keyboard.press("Escape");
    await expect(popup).not.toBeVisible();
    await expect(trigger).toBeFocused();
  } finally {
    await page.request.delete(`/api/lists/${list.list_key}`);
  }
});
