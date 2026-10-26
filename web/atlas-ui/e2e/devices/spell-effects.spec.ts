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
