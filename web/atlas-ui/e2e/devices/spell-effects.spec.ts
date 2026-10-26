import { expect, test } from "./fixtures";
import { activate, noPageOverflow } from "./reading";

const selectedSpell = (rank: number) =>
  `/records/browser-spells:battleform000001?selection=${encodeURIComponent(
    JSON.stringify({
      record_key: "browser-spells:battleform000001",
      spell_rank: rank,
      embedded_path: [],
      field: null,
      passage: null,
      source_fingerprint: null,
    }),
  )}`;

test("spell links keep rank through effect reading, explanations and full-page navigation", async ({
  page,
}, info) => {
  await page.goto(selectedSpell(4));
  const header = page.locator(".record-reference__header").first();
  await expect(header.getByRole("button", { name: /: Change rank$/ })).toBeVisible();
  const headerGeometry = () =>
    header.evaluate((node) => {
      const rect = (selector: string) =>
        node.querySelector(selector)!.getBoundingClientRect();
      const kind = rect(".record-reference__kind");
      const title = rect("h2");
      const titleRow = rect(".record-reference__title");
      const actions = [
        ...node.querySelectorAll(".record-reference__actions button"),
      ].map((button) => button.getBoundingClientRect());
      return {
        kindCenter: kind.y + kind.height / 2,
        rowTop: kind.y,
        rowBottom: Math.max(kind.bottom, title.bottom, ...actions.map((a) => a.bottom)),
        titleTop: title.y,
        kindHeight: kind.height,
        titleRowBottom: titleRow.bottom,
        actions: actions.map((a) => ({ center: a.y + a.height / 2, height: a.height })),
      };
    });
  const beforeHeader = await headerGeometry();
  expect(beforeHeader.actions).toHaveLength(2);
  for (const action of beforeHeader.actions) {
    expect(Math.abs(action.center - beforeHeader.kindCenter)).toBeLessThanOrEqual(1);
    expect(action.height).toBe(34);
  }
  expect(beforeHeader.kindHeight).toBe(34);
  expect(
    Math.abs(beforeHeader.titleRowBottom - beforeHeader.rowBottom),
  ).toBeLessThanOrEqual(1);
  await expect(header.getByRole("button", { name: /: Change rank$/ })).toHaveText(
    "Spell 4",
  );
  expect(beforeHeader.titleTop).toBeLessThanOrEqual(beforeHeader.rowTop + 4);
  await activate(
    header.getByRole("button", { name: "Explore related records", exact: true }),
    info,
  );
  expect(await headerGeometry()).toEqual(beforeHeader);
  await activate(
    header.getByRole("button", { name: "Read document", exact: true }),
    info,
  );
  const bear = page.getByRole("link", { name: "Bear", exact: true });
  await expect(bear).toBeVisible();
  const href = await bear.getAttribute("href");
  expect(
    JSON.parse(new URL(href!, page.url()).searchParams.get("selection")!).spell_rank,
  ).toBe(4);
  await activate(bear, info);
  const drawer = page.getByRole("dialog", {
    name: "Spell Effect: Animal Form (Bear)",
    exact: true,
  });
  await expect(drawer).toBeVisible();
  const stats = drawer.getByRole("region", { name: "Rank 4 statistics" });
  await expect(stats).toContainText("Large");
  await expect(stats).toContainText("15");
  await expect(stats).toContainText("18 + your level");
  await expect(stats).toContainText("2d8+9 piercing");
  const hp = stats.getByRole("button", {
    name: "Heightening details for Temporary HP: 15",
    exact: true,
  });
  await hp.focus();
  await expect(page.locator(".value-explanation")).toHaveCount(0);
  // Read both rectangles atomically: Drawer opening motion is not content reflow.
  const geometry = () =>
    stats.evaluate((node) => {
      const fact = node.getBoundingClientRect();
      const host = node.closest('[role="dialog"]')!.getBoundingClientRect();
      return {
        x: fact.x - host.x,
        y: fact.y - host.y,
        width: fact.width,
        height: fact.height,
      };
    });
  // Wait for Drawer motion to settle before measuring content reflow.
  // Trial actionability preserves the keyboard activation below.
  await hp.click({ trial: true });
  const before = await geometry();
  await hp.press("Enter");
  const explanation = page.getByRole("dialog", {
    name: "Heightening details for Temporary HP",
    exact: true,
  });
  await expect(explanation).toHaveText("Base rank: 5");
  await expect(explanation).toBeFocused();
  const after = await geometry();
  for (const key of ["x", "y", "width", "height"] as const)
    expect(after[key]).toBeCloseTo(before[key], 1);
  await page.keyboard.press("Escape");
  await expect(explanation).toBeHidden();
  await expect(drawer).toBeVisible();
  await expect(hp).toBeFocused();
  const full = drawer.getByRole("link", {
    name: "Open reference full page",
    exact: true,
  });
  expect(
    JSON.parse(
      new URL((await full.getAttribute("href"))!, page.url()).searchParams.get(
        "selection",
      )!,
    ).spell_rank,
  ).toBe(4);
  await activate(full, info);
  await expect(drawer).toBeHidden();
  await expect(page.getByRole("region", { name: "Rank 4 statistics" })).toContainText(
    "2d8+9 piercing",
  );
  await noPageOverflow(page);
  await page.goBack();
  await expect(
    page.getByRole("heading", { name: "Browser Battle Form", exact: true }),
  ).toBeVisible();
});

test("higher-rank effects preserve eligible alternatives without choosing the largest", async ({
  page,
}, info) => {
  await page.goto(selectedSpell(6));
  await activate(page.getByRole("link", { name: "Enlarge", exact: true }), info);
  const drawer = page.getByRole("dialog", {
    name: "Spell Effect: Enlarge",
    exact: true,
  });
  await expect(
    drawer.getByRole("heading", { name: "Available sizes", exact: true }),
  ).toBeVisible();
  await expect(
    drawer.getByRole("heading", { name: "Large", exact: true }),
  ).toBeVisible();
  await expect(
    drawer.getByRole("heading", { name: "Huge", exact: true }),
  ).toBeVisible();
  await expect(drawer).toContainText("Reach (fixed)");
  await expect(drawer).toContainText("+4 status");
  await expect(drawer).toContainText("+2 status");
  await expect(drawer.getByRole("radio")).toHaveCount(0);
  await noPageOverflow(page);
});
