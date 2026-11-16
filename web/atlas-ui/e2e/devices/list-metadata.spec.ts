import { expect, test } from "./fixtures";
import {
  activate,
  creatureFilter,
  noPageOverflow,
  offGuard,
  sentinel,
} from "./reading";

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
      const trigger = page.getByRole("button", { name: "About" });
      await activate(trigger, info);
      const content = page.getByLabel("List tags", { exact: true });
      const popup = page.getByRole("dialog", { name: "About", exact: true });
      await expect(popup).toBeVisible();
      await expect
        .poll(async () => {
          const box = await popup.boundingBox();
          return box !== null && box.x >= -1 && box.x + box.width <= 391;
        })
        .toBe(true);
      for (const tag of tags)
        await expect(content.getByText(tag, { exact: true })).toBeAttached();
      await popup.getByRole("button", { name: "Close", exact: true }).focus();
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
    await expect(
      page
        .getByRole("region", { name: "Record preview" })
        .getByRole("button", { name: "Add to encounter", exact: true }),
    ).toBeVisible();

    const search = results.getByRole("searchbox", { name: "Search this list" });
    const back = page.getByRole("button", { name: "Results", exact: true });
    const wide = await page.evaluate(() => innerWidth >= 992);
    if (!wide) await activate(back, info);
    await expect(search).toHaveValue("Atlas");
    await expect(results.getByRole("button", { name: "Add to encounter" })).toHaveCount(
      0,
    );
    const filters = results.getByRole("button", { name: "Filters (1)", exact: true });
    const count = results.getByText("1 of 2 records", { exact: true });
    const panel = page.getByRole("dialog", { name: "Filter saved records" });
    await expect(count).toBeVisible();
    await expect(filters).toHaveAttribute("aria-expanded", "false");
    const reader = page
      .getByRole("region", { name: "Record preview" })
      .filter({ visible: true });
    const before = wide ? await reader.boundingBox() : null;
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
    const condition = drawer.getByRole("combobox", { name: "Add condition" });
    await activate(condition, info);
    await condition.fill("Record links");
    await condition.press("Enter");
    await activate(drawer.getByRole("button", { name: "Edit link" }), info);
    const linkPicker = page.getByRole("dialog", { name: "Add link condition" });
    await expect(linkPicker).toBeVisible();
    await activate(
      linkPicker.getByRole("button", { name: "Cancel", exact: true }),
      info,
    );
    await expect(linkPicker).toBeHidden();
    await activate(drawer.getByRole("button", { name: "Close", exact: true }), info);
    await expect(drawer).toBeHidden();

    await expect(panel).toBeVisible();
    await expect(panel.getByRole("button", { name: /^Links/ })).toHaveCount(0);
    await expect(panel.getByLabel("Selected kind").getByText("Creature")).toBeVisible();
    await advanced.press("Escape");
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

test("long entry notes read in the selected record pane and preserve both scroll positions", async ({
  page,
}, info) => {
  const slug = `long-note-${info.project.name}-${Date.now()}`;
  const created = await page.request.post("/api/lists", {
    data: { slug, name: "Long note review" },
  });
  expect(created.ok()).toBeTruthy();
  const { list } = await created.json();
  const source = [
    "The sentinel remembers anyone who relights the lantern.",
    "## At the table",
    ...Array.from(
      { length: 42 },
      (_value, i) =>
        `- Preparation ${i + 1}. Ask the party to choose a route and assign someone to watch the gate.`,
    ),
    "If the light goes out, use [Off-Guard](record:browser-conditions:offguard00000001).",
  ].join("\n\n");
  try {
    const added = await page.request.post(`/api/lists/${list.list_key}/items`, {
      data: { record_ref: sentinel, note: source },
    });
    expect(added.ok()).toBeTruthy();
    const path = `/lists/${slug}/records/${sentinel}`;
    await page.goto(path);
    const footer = page.getByRole("region", { name: "List entry note" });
    const read = footer.getByRole("button", { name: "Read full note" });
    const rules = page.locator(".list-workspace .reference-reader__scroll");
    const offGuardLink = footer.getByRole("link", { name: "Off-Guard", exact: true });
    await expect(read).toBeVisible();
    await expect(offGuardLink).toHaveCount(0);
    await expect(
      rules.getByRole("heading", { name: "Atlas Sentinel", exact: true }),
    ).toBeVisible();
    const rulesPosition = await rules.evaluate((node) => {
      node.scrollTop = node.scrollHeight;
      return node.scrollTop;
    });
    expect(rulesPosition).toBeGreaterThan(0);
    await page.screenshot({ path: info.outputPath("long-note-footer.png") });

    await activate(read, info);
    const noteHeading = footer.getByRole("heading", {
      name: "Atlas Sentinel",
      exact: true,
    });
    await expect(noteHeading).toBeFocused();
    await expect(rules).toBeHidden();
    await expect(footer.getByRole("heading", { name: "At the table" })).toBeVisible();
    await expect(page).toHaveURL((url) => url.pathname === path);
    await page.screenshot({ path: info.outputPath("full-list-note.png") });

    const results = page.getByRole("button", { name: "Results", exact: true });
    if (await results.isVisible()) {
      await activate(results, info);
      const row = page
        .getByRole("region", { name: "Saved records" })
        .locator(`[data-record-key="${sentinel}"]`);
      await expect(row).toBeFocused();
      await activate(row, info);
      await expect(noteHeading).toBeFocused();
    }
    const noteBody = footer.getByRole("region", { name: "Note text" });
    await noteBody.focus();
    await page.keyboard.press("PageDown");
    await expect
      .poll(() => noteBody.evaluate((node) => node.scrollTop))
      .toBeGreaterThan(0);
    await offGuardLink.scrollIntoViewIfNeeded();
    const notePosition = await noteBody.evaluate((node) => node.scrollTop);
    expect(notePosition).toBeGreaterThan(0);
    await activate(offGuardLink, info);
    await expect(page.getByRole("dialog", { name: "Off-Guard" })).toBeVisible();
    await expect(page.locator(".record-preview-drawer")).toHaveCount(1);
    await activate(page.getByRole("button", { name: "Close reference preview" }), info);
    await expect(offGuardLink).toBeFocused();
    await expect
      .poll(() => noteBody.evaluate((node) => node.scrollTop))
      .toBeCloseTo(notePosition, 0);

    await activate(footer.getByRole("button", { name: "Back to record" }), info);
    await expect(read).toBeFocused();
    await expect
      .poll(() => rules.evaluate((node) => node.scrollTop))
      .toBeCloseTo(rulesPosition, 0);

    await activate(read, info);
    await activate(footer.getByRole("button", { name: "Edit", exact: true }), info);
    const draft = footer.getByRole("textbox", { name: "Note", exact: true });
    await expect(draft).toHaveValue(source);
    await expect(footer.getByRole("button", { name: "Collapse editor" })).toHaveCount(
      0,
    );
    await draft.fill("");
    await activate(footer.getByRole("button", { name: "Save", exact: true }), info);
    await expect(footer.getByRole("button", { name: "Add note" })).toBeFocused();
    await expect(rules).toBeVisible();
    await noPageOverflow(page);
  } finally {
    await page.request.delete(`/api/lists/${list.list_key}`);
  }
});

test("expanding a new entry note retains the editor, caret and preview choice", async ({
  page,
}, info) => {
  const slug = `expand-note-${info.project.name}-${Date.now()}`;
  const created = await page.request.post("/api/lists", {
    data: { slug, name: "Expanded editor review" },
  });
  expect(created.ok()).toBeTruthy();
  const { list } = await created.json();
  try {
    const added = await page.request.post(`/api/lists/${list.list_key}/items`, {
      data: { record_ref: sentinel },
    });
    expect(added.ok()).toBeTruthy();
    await page.goto(`/lists/${slug}/records/${sentinel}`);
    const note = page.getByRole("region", { name: "List entry note" });
    const expand = note.getByRole("button", { name: "Expand editor" });
    const collapse = note.getByRole("button", { name: "Collapse editor" });
    await activate(note.getByRole("button", { name: "Add note" }), info);
    const editor = note.getByRole("textbox", { name: "Note", exact: true });
    const text = "Keep this **draft** while the list stays open. ".repeat(24).trimEnd();
    await editor.fill(text);
    await editor.evaluate((node: HTMLTextAreaElement) => node.setSelectionRange(5, 9));
    const original = await editor.elementHandle();
    await expect(expand).toBeVisible();
    await activate(expand, info);
    await expect(collapse).toBeFocused();
    await expect(editor).toHaveValue(text);
    expect(await editor.evaluate((node, before) => node === before, original)).toBe(
      true,
    );
    expect(
      await editor.evaluate((node: HTMLTextAreaElement) => [
        node.selectionStart,
        node.selectionEnd,
      ]),
    ).toEqual([5, 9]);
    await page.screenshot({ path: info.outputPath("expanded-list-note-editor.png") });

    const results = page.getByRole("button", { name: "Results", exact: true });
    if (await results.isVisible()) {
      await editor.focus();
      await activate(results, info);
      await activate(
        page
          .getByRole("region", { name: "Saved records" })
          .locator(`[data-record-key="${sentinel}"]`),
        info,
      );
      await expect(editor).toBeFocused();
    }
    const resize = page.viewportSize()!;
    await editor.focus();
    try {
      await page.setViewportSize({
        width: resize.width < 992 ? 1110 : 390,
        height: 420,
      });
      await expect(editor).toBeFocused();
      await expect(collapse).toBeVisible();
      await noPageOverflow(page);
    } finally {
      await page.setViewportSize(resize);
    }

    const preview = note.getByText("Preview", { exact: true });
    await activate(preview, info);
    await expect(note.locator(".markdown-editor__preview strong").first()).toHaveText(
      "draft",
    );
    await activate(collapse, info);
    await expect(expand).toBeFocused();
    await expect(note.locator(".markdown-editor__preview strong").first()).toHaveText(
      "draft",
    );
    await activate(note.getByText("Write", { exact: true }), info);
    await expect(editor).toHaveValue(text);
    await activate(note.getByRole("button", { name: "Cancel", exact: true }), info);
    await expect(note.getByRole("button", { name: "Add note" })).toBeFocused();
  } finally {
    await page.request.delete(`/api/lists/${list.list_key}`);
  }
});

test("note overflow follows the rendered layout and a removed open note restores list focus", async ({
  page,
}, info) => {
  test.skip(info.project.name !== "desktop-chromium", "Changes the desktop viewport");
  const slug = `reflow-note-${Date.now()}`;
  const created = await page.request.post("/api/lists", {
    data: { slug, name: "Note reflow review" },
  });
  expect(created.ok()).toBeTruthy();
  const { list } = await created.json();
  const text = "Check the lantern by the western gate. ".repeat(5);
  const noteSource = `${text}[Signal](https://example.com/${"a".repeat(700)})`;
  try {
    for (const data of [
      { record_ref: sentinel, note: noteSource },
      { record_ref: offGuard },
    ]) {
      const added = await page.request.post(`/api/lists/${list.list_key}/items`, {
        data,
      });
      expect(added.ok()).toBeTruthy();
    }
    await page.goto(`/lists/${slug}/records/${sentinel}`);
    const note = page.getByRole("region", { name: "List entry note" });
    const signal = note.getByRole("link", { name: "Signal", exact: true });
    const read = note.getByRole("button", { name: "Read full note" });
    await expect(signal).toBeVisible();
    await expect(read).toHaveCount(0);
    await signal.focus();
    await page.setViewportSize({ width: 390, height: 420 });
    await expect(read).toBeFocused();
    await expect(signal).toHaveCount(0);
    const bounds = await read.boundingBox();
    const region = await note.boundingBox();
    expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(region!.y + region!.height);
    await read.press("Enter");
    await expect(signal).toBeVisible();
    await page.setViewportSize({ width: 1440, height: 900 });
    const back = note.getByRole("button", { name: "Back to record" });
    await activate(back, info);
    await expect(note.getByRole("button", { name: "Edit", exact: true })).toBeFocused();
    await expect(read).toHaveCount(0);

    await page.setViewportSize({ width: 390, height: 420 });
    await activate(read, info);
    await page.setViewportSize({ width: 1440, height: 900 });
    const results = page.getByRole("region", { name: "Saved records" });
    await activate(
      results.getByRole("button", { name: "Remove Atlas Sentinel" }),
      info,
    );
    await activate(page.getByRole("button", { name: "Remove", exact: true }), info);
    await expect(note).toBeHidden();
    await expect(results.locator(`[data-record-key="${offGuard}"]`)).toBeFocused();
  } finally {
    await page.request.delete(`/api/lists/${list.list_key}`);
  }
});
