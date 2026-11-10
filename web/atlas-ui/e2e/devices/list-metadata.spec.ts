import { expect, test } from "./fixtures";
import { activate, creatureFilter, offGuard, sentinel } from "./reading";

test(
  "measured tags hide immediately and their phone disclosure stays in the viewport",
  { tag: "@engine" },
  async ({ page }, info) => {
    const slug = `browser-metadata-${Date.now()}`;
    const name = `Browser metadata ${slug}`;
    const tags = Array.from(
      { length: 20 },
      (_, index) => `Long review tag ${String(index + 1).padStart(2, "0")}`,
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
      const trigger = page.getByRole("button", { name: "About this list" });
      await activate(trigger, info);
      const content = page.getByLabel("List tags", { exact: true });
      const popup = page.getByRole("tooltip").filter({ has: content });
      await expect(popup).toBeVisible();
      await expect
        .poll(async () => {
          const box = await popup.boundingBox();
          return box !== null && box.x >= -1 && box.x + box.width <= 391;
        })
        .toBe(true);
      for (const tag of tags)
        await expect(content.getByText(tag, { exact: true })).toBeAttached();
      // Touch activation does not establish keyboard focus in WebKit.
      await trigger.focus();
      await page.keyboard.press("Escape");
      await expect(popup).not.toBeVisible();
      await expect(trigger).toBeFocused();
    } finally {
      await page.request.delete(`/api/lists/${list.list_key}`);
    }
  },
);

test("saved-list controls stay local and retain search while clearing structured filters", async ({
  page,
}, info) => {
  const slug = `pane-controls-${Date.now()}`;
  const created = await page.request.post("/api/lists", {
    data: { slug, name: "Pane controls review" },
  });
  expect(created.ok()).toBeTruthy();
  const { list } = await created.json();
  try {
    for (const record_ref of [sentinel, offGuard]) {
      const added = await page.request.post(`/api/lists/${slug}/items`, {
        data: { record_ref },
      });
      expect(added.ok()).toBeTruthy();
    }
    await page.goto(
      `/lists/${slug}/records/${sentinel}?q=Atlas&filter=${creatureFilter}`,
    );
    const results = page.getByRole("region", { name: "Saved records" });
    const search = results.getByRole("searchbox", { name: "Search this list" });
    const back = page.getByRole("button", { name: "Results", exact: true });
    if (await page.evaluate(() => innerWidth < 992)) {
      await activate(back, info);
    }
    await expect(search).toHaveValue("Atlas");
    const filters = results.getByRole("button", { name: "Filters (1)", exact: true });
    const count = results.getByText("1 of 2 records", { exact: true });
    const panel = page.getByRole("dialog", { name: "Filter saved records" });
    await expect(count).toBeVisible();
    await expect(filters).toHaveAttribute("aria-expanded", "false");
    const reader = page
      .getByRole("region", { name: "Record preview" })
      .filter({ visible: true });
    const before = await reader.boundingBox();
    await activate(filters, info);
    await expect(panel).toBeVisible();
    if (before) {
      const panelBox = (await panel.boundingBox())!;
      expect(panelBox.y).toBeGreaterThanOrEqual((await count.boundingBox())!.y);
      expect(panelBox.x + panelBox.width).toBeLessThanOrEqual(before.x + 1);
      expect(await reader.boundingBox()).toEqual(before);
    }

    const advanced = panel.getByRole("button", { name: "Advanced filters" });
    const clearBox = (await panel
      .getByRole("button", { name: "Clear filters" })
      .boundingBox())!;
    await activate(advanced, info);
    const drawer = page.getByRole("dialog", { name: "Advanced filters" });
    await expect(drawer).toBeVisible();
    if (before) {
      await expect
        .poll(() =>
          drawer.evaluate(
            (node, { x, y }) => node.contains(document.elementFromPoint(x, y)),
            { x: clearBox.x + clearBox.width / 2, y: clearBox.y + clearBox.height / 2 },
          ),
        )
        .toBe(true);
    }
    await activate(drawer.getByRole("button", { name: "Close", exact: true }), info);
    await expect(drawer).toBeHidden();

    const links = panel.getByRole("button", { name: /^Links/ });
    await activate(links, info);
    const linkPicker = page.getByRole("dialog", { name: "Add link condition" });
    await expect(linkPicker).toBeVisible();
    await activate(
      linkPicker.getByRole("button", { name: "Cancel", exact: true }),
      info,
    );
    await expect(linkPicker).toBeHidden();
    await expect(panel).toBeVisible();
    await links.press("Escape");
    await expect(panel).toBeHidden();

    await activate(filters, info);
    await activate(panel.getByRole("button", { name: "Clear filters" }), info);
    await expect(search).toHaveValue("Atlas");
    await expect(count).toBeVisible();
    await expect(
      results.getByRole("button", { name: "Filters", exact: true }),
    ).toBeVisible();
    await expect(
      results.getByRole("button", { name: "Sort: Saved order" }),
    ).toBeVisible();
    await activate(panel.getByRole("button", { name: "Done" }), info);
    await expect(panel).toBeHidden();
    await expect(page).toHaveURL(/q=Atlas/);
    expect(new URL(page.url()).searchParams.has("filter")).toBe(false);

    if (before) {
      await page.goto(`/lists/${slug}?q=Atlas`);
      const trigger = results.getByRole("button", { name: "Filters", exact: true });
      await activate(trigger, info);
      const minimum = panel.getByRole("textbox", { name: "Level / Rank minimum" });
      await minimum.focus();
      await page.setViewportSize({ width: 390, height: 360 });
      await expect(trigger).toHaveAttribute("aria-expanded", "true");
      await expect(panel).toBeFocused();
      await activate(panel.getByRole("button", { name: "Done" }), info);
      await expect(panel).toBeHidden();
      await expect(trigger).toBeFocused();
      const record = results.getByRole("link", { name: /^Atlas Sentinel/ });
      await record.scrollIntoViewIfNeeded();
      await expect(record).toBeInViewport();
    }
  } finally {
    await page.request.delete(`/api/lists/${list.list_key}`);
  }
});
