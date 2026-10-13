import { expect, test } from "@playwright/test";

const creatureKey = "stolen-fate-bestiary:5jXQKIwhlOfs9F6e";
const hazardKey = "pfs-season-3-bestiary:wun5VzDIMSZCVLuI";

test("creature lore precedes mechanics and selected casting retains local membership", async ({
  page,
}) => {
  await page.goto("/records/" + encodeURIComponent(creatureKey));
  const lore = page.locator(".record-reference__flavor");
  await expect(lore).toContainText("harrowkin");
  const strikes = page.getByRole("heading", { name: "Strikes", exact: true });
  await expect(strikes).toBeVisible();
  const flavorBox = await lore.boundingBox();
  const strikesBox = await strikes.boundingBox();
  expect(flavorBox!.y).toBeLessThan(strikesBox!.y);

  const selection = {
    record_key: creatureKey,
    owners: [{ collection: "/items", identity: { Stable: "aLhWdKoUI0wcntZE" } }],
    field: null,
    passage: null,
    source_fingerprint: null,
  };
  await page.goto(
    "/records/" +
      encodeURIComponent(creatureKey) +
      "?selection=" +
      encodeURIComponent(JSON.stringify(selection)),
  );
  await expect(
    page.getByRole("heading", { name: "Occult Innate Spells", exact: true }),
  ).toBeVisible();
  await expect(page.getByText("Ghost Sound", { exact: true })).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Return to Fabled Harrowkin" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Description", exact: true }),
  ).toHaveCount(0);
});

test("hazard ability reads inline and returns to its exact parent reading position", async ({
  page,
}) => {
  const path = "/records/" + hazardKey;
  await page.goto(path);
  await expect(page.getByRole("heading", { name: "Waking Nightmare" })).toBeVisible();
  const ability = page
    .locator(".owned-ability-reading__entry")
    .filter({ has: page.getByRole("heading", { name: /^Seize\b/ }) });
  const content = ability.locator(".prepared-content");
  await expect(content).toContainText("Trigger");
  await expect(content).toContainText("Effect");
  expect(new URL(page.url()).pathname).toBe("/records/" + hazardKey);
  await expect(ability.locator("[data-atlas-action-glyph]")).toHaveCount(1);
  const fullPage = page.getByRole("link", {
    name: "Open Seize full reference",
    exact: true,
  });
  const target = new URL((await fullPage.getAttribute("href"))!, page.url());
  expect(target.pathname).toBe("/records/" + hazardKey);
  const selection = JSON.parse(target.searchParams.get("selection")!);
  expect(selection.record_key).toBe(hazardKey);
  expect(selection.owners).toEqual([
    { collection: "/items", identity: { Stable: "9awhcUUChblK5dxi" } },
  ]);
  await fullPage.click();
  await expect(page.getByRole("heading", { name: /^Seize\b/ })).toBeVisible();
  await expect(page).toHaveURL(target.href);
  const owner = page.getByRole("link", { name: "Return to Waking Nightmare" });
  const parentUrl = new URL((await owner.getAttribute("href"))!, page.url());
  expect(JSON.parse(parentUrl.searchParams.get("landing")!).owners).toEqual(
    selection.owners,
  );
  expect(parentUrl.searchParams.has("selection")).toBe(false);
  await owner.click();
  await expect(page.getByRole("heading", { name: "Waking Nightmare" })).toBeVisible();
  await expect(page.getByRole("heading", { name: /^Seize\b/ })).toBeFocused();
  await page.goBack();
  await expect(page.getByRole("heading", { name: /^Seize\b/ })).toBeVisible();
  await page.goForward();
  await expect(page.getByRole("heading", { name: /^Seize\b/ })).toBeFocused();
});

test("GM Core Hidden Pit is available in the preview through normal search", async ({
  page,
}) => {
  await page.goto("/search?q=hidden%20pit");
  const result = page
    .locator("[data-record-key]")
    .filter({ hasText: "Hidden Pit" })
    .first();
  await expect(result).toBeVisible();
  await result.click();
  await expect(
    page.getByRole("heading", { name: "Hidden Pit", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("heading", { name: /^Pitfall\b/ })).toBeVisible();
});

test("owned reference return retains Similar, canonical filters, selected result and native landing", async ({
  page,
  context,
}) => {
  const filter = {
    kind: "in",
    clause_id: "hazard-kind",
    field: "record.kind",
    values: ["hazard"],
  };
  const params = new URLSearchParams({
    similar: "pathfinder-bestiary:LHHgGSs0ELCR4CYK",
    filter: JSON.stringify(filter),
  });
  await page.goto("/search/records/" + hazardKey + "?" + params.toString());
  const permalink = page.getByRole("link", {
    name: "Open Seize full reference",
  });
  await expect(permalink).toBeVisible();
  const selectedHref = await permalink.getAttribute("href");
  await permalink.click();
  await expect(page.getByRole("heading", { name: /^Seize\b/ })).toBeVisible();
  const owner = page.getByRole("link", { name: "Return to Waking Nightmare" });
  const href = (await owner.getAttribute("href"))!;
  const target = new URL(href, page.url());
  expect(target.pathname).toBe("/search/records/" + hazardKey);
  expect(target.searchParams.get("similar")).toBe(params.get("similar"));
  expect(JSON.parse(target.searchParams.get("filter")!)).toEqual(filter);
  expect(target.searchParams.has("selection")).toBe(false);
  expect(JSON.parse(target.searchParams.get("landing")!).owners).toEqual(
    JSON.parse(new URL(selectedHref!, page.url()).searchParams.get("selection")!)
      .owners,
  );
  // The href is complete for browser-native new-tab navigation, independent of history state.
  const other = await context.newPage();
  await other.goto(target.href);
  await expect(other.getByRole("heading", { name: /^Seize\b/ })).toBeFocused();
  await other.close();
  await owner.click();
  await expect(page.getByRole("heading", { name: /^Seize\b/ })).toBeFocused();
  await expect(page.getByText("Similar to Ghoul", { exact: true })).toBeVisible();
});
