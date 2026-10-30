import { expect, test, type Page } from "@playwright/test";

async function documentIdentity(page: Page) {
  return page.evaluate(() => ({
    marker: Reflect.get(window, "atlasNavigationDocument"),
    timeOrigin: performance.timeOrigin,
  }));
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    Reflect.set(window, "atlasNavigationDocument", crypto.randomUUID());
  });
});

test("encounter titles and focused rows navigate once in the same document", async ({
  page,
}) => {
  const name = `Navigation continuity ${Date.now()}`;
  const created = await page.request.post("/api/encounters", { data: { name } });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  const href = `/encounters/${encounter.slug}`;
  try {
    await page.goto("/encounters");
    const title = page.getByRole("link", { name, exact: true });
    await expect(title).toHaveAttribute("href", href);
    await page.waitForLoadState("networkidle");
    const identity = await documentIdentity(page);
    const bootstrap: string[] = [];
    page.on("request", (request) => {
      if (/\/api\/(compatibility|readiness)$/.test(new URL(request.url()).pathname))
        bootstrap.push(request.url());
    });
    for (const activation of ["click", "link Enter", "row Enter"]) {
      if (activation === "click") await title.click();
      else if (activation === "link Enter") await title.press("Enter");
      else await title.locator("xpath=ancestor::tr").press("Enter");
      await expect(page).toHaveURL(new RegExp(`${href}$`));
      await expect(page.getByRole("heading", { name, exact: true })).toBeVisible();
      expect(await documentIdentity(page)).toEqual(identity);
      await page.goBack();
      await expect(page).toHaveURL(/\/encounters$/);
      await expect(title).toBeVisible();
    }
    expect(bootstrap).toEqual([]);
    // The adjacent edit control keeps its existing destination and workflow.
    const edit = page.getByRole("link", { name: `Edit ${name}`, exact: true });
    await expect(edit).toHaveAttribute("href", `${href}/edit`);
    await edit.click();
    await expect(page.getByRole("textbox", { name: /Name$/ })).toHaveValue(name);
    expect(await documentIdentity(page)).toEqual(identity);
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
  }
});

test("encounter title and edit hrefs retain native new-tab and middle-click activation", async ({
  page,
  context,
  browserName,
}) => {
  const name = `Native encounter links ${Date.now()}`;
  const created = await page.request.post("/api/encounters", { data: { name } });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  try {
    await page.goto("/encounters");
    let identity = await documentIdentity(page);
    for (const [label, suffix, middle] of [
      [name, "", false],
      [name, "", true],
      [`Edit ${name}`, "/edit", false],
    ] as const) {
      const link = page.getByRole("link", { name: label, exact: true });
      const href = `/encounters/${encounter.slug}${suffix}`;
      await expect(link).toHaveAttribute("href", href);
      if (middle && browserName === "webkit") {
        // This headless engine follows the baseline native anchor in the current
        // tab on middle-click. Preserve that default; Chromium opens a new tab.
        await link.click({ button: "middle" });
        await expect(page).toHaveURL(new RegExp(`${href}$`));
        await expect(page.getByRole("heading", { name, exact: true })).toBeVisible();
        expect((await documentIdentity(page)).timeOrigin).not.toBe(identity.timeOrigin);
        await page.goBack();
        await expect(link).toBeVisible();
        identity = await documentIdentity(page);
        continue;
      }
      const popup = context.waitForEvent("page");
      if (middle) await link.click({ button: "middle" });
      else await link.click({ modifiers: ["ControlOrMeta"] });
      const opened = await popup;
      try {
        await expect(opened).toHaveURL(new RegExp(`${href}$`));
        await opened.waitForLoadState("domcontentloaded");
        expect((await documentIdentity(opened)).timeOrigin).not.toBe(
          identity.timeOrigin,
        );
      } finally {
        await opened.close();
      }
      await expect(page).toHaveURL(/\/encounters$/);
      expect(await documentIdentity(page)).toEqual(identity);
    }
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
  }
});

test("encounter title trips preserve Search criteria, page, reader, drafts and scroll", async ({
  page,
}) => {
  const name = `Retained Search ${Date.now()}`;
  const created = await page.request.post("/api/encounters", { data: { name } });
  expect(created.ok()).toBeTruthy();
  const { encounter } = await created.json();
  try {
    // This real-corpus suite uses the pinned source's long Ghoul reader independently
    // of the submitted criteria, which the Search owner deliberately supports.
    await page.goto(
      "/search/records/pathfinder-bestiary:LHHgGSs0ELCR4CYK?q=magic&page=2&search_profile=bge-base",
    );
    await expect(
      page.locator(".reference-result [data-record-key]").first(),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Ghoul", exact: true }),
    ).toBeVisible();
    const input = page.getByRole("searchbox", { name: "Search records", exact: true });
    await input.fill("unfinished search intent");
    const range = page.getByRole("button", { name: /Edit Level \/ rank range/i });
    await range.click();
    await page.getByRole("textbox", { name: /Level \/ rank minimum/i }).fill("-");
    await page.keyboard.press("Escape");
    const url = page.url();
    const identity = await documentIdentity(page);
    const positions = await page
      .locator(".reference-results__scroll, .reference-reader__scroll")
      .evaluateAll((nodes) =>
        nodes.map((node) => {
          node.scrollTop = Math.min(180, node.scrollHeight - node.clientHeight);
          node.dispatchEvent(new Event("scroll"));
          return node.scrollTop;
        }),
      );
    expect(positions.length).toBe(2);
    expect(positions.every((position) => position > 0)).toBe(true);
    for (const returnPath of ["Search control", "Brand", "Back"]) {
      await page.getByRole("link", { name: "Encounters", exact: true }).first().click();
      await page.getByRole("link", { name, exact: true }).click();
      await expect(page.getByRole("heading", { name, exact: true })).toBeVisible();
      if (returnPath === "Back") {
        await page.goBack();
        await page.goBack();
      } else {
        const search = page
          .getByRole("link", {
            name: returnPath === "Brand" ? "PF2e Atlas" : "Search",
            exact: true,
          })
          .first();
        const target = new URL(url);
        await expect(search).toHaveAttribute(
          "href",
          target.pathname + target.search + target.hash,
        );
        const opened = page.context().waitForEvent("page");
        await search.click({ modifiers: ["ControlOrMeta"] });
        const copied = await opened;
        try {
          await expect(copied).toHaveURL(url);
          await expect(
            copied.getByRole("searchbox", { name: "Search records", exact: true }),
          ).toHaveValue("magic");
          await expect(
            copied.getByRole("heading", { name: "Ghoul", exact: true }),
          ).toBeVisible();
        } finally {
          await copied.close();
        }
        await expect(input).not.toBeVisible();
        if (returnPath === "Brand") await search.press("Enter");
        else await search.click();
      }
      await expect(page).toHaveURL(url);
      await expect(input).toHaveValue("unfinished search intent");
      await expect(
        page.getByRole("heading", { name: "Ghoul", exact: true }),
      ).toBeVisible();
      expect(await documentIdentity(page)).toEqual(identity);
      await expect
        .poll(() =>
          page
            .locator(".reference-results__scroll, .reference-reader__scroll")
            .evaluateAll((nodes) => nodes.map((node) => node.scrollTop)),
        )
        .toEqual(positions);
      await range.click();
      await expect(
        page.getByRole("textbox", { name: /Level \/ rank minimum/i }),
      ).toHaveValue("-");
      await page.keyboard.press("Escape");
    }
  } finally {
    await page.request.delete(`/api/encounters/${encounter.encounter_key}`);
  }
});
