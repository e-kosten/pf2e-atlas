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
    page.getByRole("button", { name: "From Fabled Harrowkin" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Description", exact: true }),
  ).toHaveCount(0);
});

test("hazard ability expands without losing its parent and preserves exact full-page owner", async ({
  page,
}) => {
  const path = "/records/" + encodeURIComponent(hazardKey);
  await page.goto(path);
  await expect(page.getByRole("heading", { name: "Waking Nightmare" })).toBeVisible();
  await page.getByRole("button", { name: /^Seize\b/ }).click();
  const content = page.locator(".actor-ability__detail .prepared-content");
  await expect(content).toContainText("Trigger");
  await expect(content).toContainText("Effect");
  await expect(page).toHaveURL(new RegExp(encodeURIComponent(hazardKey) + "$"));
  const fullPage = page.getByRole("link", {
    name: "Open Seize full page",
    exact: true,
  });
  const target = new URL((await fullPage.getAttribute("href"))!, page.url());
  const selection = JSON.parse(target.searchParams.get("selection")!);
  expect(selection.record_key).toBe(hazardKey);
  expect(selection.owners).toEqual([
    { collection: "/items", identity: { Stable: "9awhcUUChblK5dxi" } },
  ]);
  await fullPage.click();
  await expect(page.getByRole("heading", { name: "Seize", exact: true })).toBeVisible();
  await expect(page).toHaveURL(target.href);
  await page.goBack();
  await expect(page.getByRole("heading", { name: "Waking Nightmare" })).toBeVisible();
});

test("GM Core Hidden Pit is available in the preview through normal search", async ({
  page,
}) => {
  await page.goto("/search?q=hidden%20pit");
  const result = page.getByRole("link", { name: "Hidden Pit", exact: true }).first();
  await expect(result).toBeVisible();
  await result.click();
  await expect(
    page.getByRole("heading", { name: "Hidden Pit", exact: true }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: /^Pitfall\b/ })).toBeVisible();
});
